mod estrategias_conflictos;
mod region;

use region::Region;
use std::{
    path::{self, Path, PathBuf},
    rc::Rc,
};

use crate::{
    io::{self, escribir_bytes, leer_a_string, rm_directorio},
    tipos_de_dato::{
        comandos::merge::{
            estrategias_conflictos::resolver_merge_len_2, region::unificar_regiones,
        },
        logger::Logger,
        objetos::{commit::CommitObj, tree::Tree},
    },
    utilidades_de_compresion::descomprimir_objeto,
    utilidades_index::{escribir_index, leer_index, ObjetoIndex},
};

use self::estrategias_conflictos::{conflicto_len_3, conflicto_len_4};

use super::{
    cat_file,
    commit::Commit,
    log::Log,
    write_tree::{self, conseguir_arbol_from_hash_commit},
};

pub struct Merge {
    pub logger: Rc<Logger>,
    pub branch_actual: String,
    pub branch_a_mergear: String,
}
#[derive(Debug, Clone)]

pub enum LadoConflicto {
    Head,
    Entrante,
}
#[derive(Debug, Clone)]

pub enum DiffType {
    Added(String),
    Removed(String),
    Unchanged(String),
}
type ConflictoAtomico = Vec<(DiffType, LadoConflicto)>;

type DiffGrid = Vec<Vec<usize>>;

impl Merge {
    pub fn from(args: &mut Vec<String>, logger: Rc<Logger>) -> Result<Merge, String> {
        if args.len() != 1 {
            return Err("Cantidad de argumentos invalida".to_string());
        }
        let branch_a_mergear = args.pop().unwrap();
        let branch_actual = Commit::obtener_branch_actual()?;
        Ok(Merge {
            logger,
            branch_actual,
            branch_a_mergear,
        })
    }

    fn obtener_arbol_commit_actual(branch: String) -> Result<Tree, String> {
        let head_commit = io::leer_a_string(format!(".gir/refs/heads/{}", branch))?;
        let hash_tree_padre =
            conseguir_arbol_from_hash_commit(&head_commit, String::from(".gir/objects/"));
        Ok(Tree::from_hash(hash_tree_padre, PathBuf::from("."))?)
    }

    fn obtener_commit_base_entre_dos_branches(&self) -> Result<String, String> {
        let hash_commit_actual = Commit::obtener_hash_del_padre_del_commit()?;
        let hash_commit_a_mergear = Self::obtener_commit_de_branch(&self.branch_a_mergear)?;

        let commit_obj_actual = CommitObj::from_hash(hash_commit_actual)?;
        let commit_obj_a_mergear = CommitObj::from_hash(hash_commit_a_mergear)?;

        let commits_branch_actual = Log::obtener_listas_de_commits(commit_obj_actual)?;
        let commits_branch_a_mergear = Log::obtener_listas_de_commits(commit_obj_a_mergear)?;

        for commit_actual in commits_branch_actual {
            for commit_branch_merge in commits_branch_a_mergear.clone() {
                if commit_actual.hash == commit_branch_merge.hash {
                    return Ok(commit_actual.hash);
                }
            }
        }
        Err("No se encontro un commit base entre las dos ramas".to_string())
    }

    fn obtener_diffs_entre_dos_objetos(
        hash_objeto1: String,
        hash_objeto2: String,
    ) -> Result<Vec<(usize, DiffType)>, String> {
        let (_, contenido1) = cat_file::obtener_contenido_objeto(hash_objeto1)?;
        let (_, contenido2) = cat_file::obtener_contenido_objeto(hash_objeto2)?;
        let contenido1_splitteado = contenido1.split('\n').collect::<Vec<&str>>();
        let contenido2_splitteado = contenido2.split('\n').collect::<Vec<&str>>();
        let diff = Self::obtener_diff(contenido1_splitteado, contenido2_splitteado);
        Ok(diff)
    }

    fn computar_lcs_grid(texto1: &Vec<&str>, texto2: &Vec<&str>) -> DiffGrid {
        let longitud1 = texto1.len();
        let longitud2 = texto2.len();

        let mut matriz_lcs = vec![vec![0; longitud2 + 1]; longitud1 + 1];

        for i in 0..=longitud1 {
            for j in 0..=longitud2 {
                if i == 0 || j == 0 {
                    matriz_lcs[i][j] = 0;
                } else if texto1[i - 1] == texto2[j - 1] {
                    matriz_lcs[i][j] = 1 + matriz_lcs[i - 1][j - 1];
                } else {
                    matriz_lcs[i][j] = std::cmp::max(matriz_lcs[i][j - 1], matriz_lcs[i - 1][j]);
                }
            }
        }
        matriz_lcs
    }

    fn cantidad_de_anteriores_solo_con_remove(
        resultados: &Vec<(usize, DiffType)>,
        i: usize,
    ) -> usize {
        let mut cantidad = 0;
        let mut i = i;
        let mut linea = resultados[i].0;
        let mut tiene_solo_remove = true;
        while i != 0 {
            if linea != resultados[i].0 {
                if tiene_solo_remove {
                    cantidad += 1;
                }
                linea = resultados[i].0;
                tiene_solo_remove = true;
            }
            match resultados[i].1 {
                DiffType::Removed(_) => i -= 1,
                _ => {
                    tiene_solo_remove = false;
                    i -= 1;
                }
            }
        }
        if tiene_solo_remove {
            cantidad += 1;
        }
        cantidad
    }

    /// los resultados pueden venir con lineas donde solo hay removes, por lo que hay que mover el usize los add
    /// en los casos donde la linea anterior solo tiene removes
    /// TODO: hacer mas eficiente con PD
    fn reindexar_resultados(resultados: &mut Vec<(usize, DiffType)>) {
        for i in 0..resultados.len() {
            if let DiffType::Added(_) = resultados[i].1 {
                let cantidad_anteriores_solo_con_remove =
                    Self::cantidad_de_anteriores_solo_con_remove(&resultados, i);
                resultados[i].0 -= cantidad_anteriores_solo_con_remove;
            }
        }
    }

    //en texto 1 debe ir la base
    fn obtener_diff(texto1: Vec<&str>, texto2: Vec<&str>) -> Vec<(usize, DiffType)> {
        let diff_grid = Self::computar_lcs_grid(&texto1, &texto2);
        let mut i = texto1.len();
        let mut j = texto2.len();
        let mut resultado_diff: Vec<(usize, DiffType)> = Vec::new();

        while i != 0 || j != 0 {
            if i == 0 {
                resultado_diff.push((j, DiffType::Added(texto2[j - 1].trim().to_string())));
                j -= 1;
            } else if j == 0 {
                resultado_diff.push((i, DiffType::Removed(texto1[i - 1].trim().to_string())));
                i -= 1;
            } else if texto1[i - 1] == texto2[j - 1] {
                resultado_diff.push((i, DiffType::Unchanged(texto1[i - 1].trim().to_string())));
                i -= 1;
                j -= 1;
            } else if diff_grid[i - 1][j] <= diff_grid[i][j - 1] {
                resultado_diff.push((i, DiffType::Added(texto2[j - 1].trim().to_string())));
                j -= 1;
            } else {
                resultado_diff.push((i, DiffType::Removed(texto1[i - 1].trim().to_string())));
                i -= 1
            }
        }
        resultado_diff.reverse();
        Self::reindexar_resultados(&mut resultado_diff);
        resultado_diff
    }

    fn hay_conflicto(posibles_conflictos: &Vec<(DiffType, LadoConflicto)>) -> bool {
        if posibles_conflictos.len() == 1 {
            if let DiffType::Added(_) = posibles_conflictos[0].0 {
                return false;
            }
        }

        posibles_conflictos.iter().all(|(diff, _)| match diff {
            DiffType::Unchanged(_) => false,
            _ => true,
        })
    }

    fn resolver_conflicto<'a>(conflicto: &'a ConflictoAtomico, linea_base: &'a str) -> Region {
        if conflicto.len() == 4 {
            conflicto_len_4(conflicto)
        } else {
            conflicto_len_3(conflicto, linea_base)
        }
    }

    fn mergear_archivos(
        diff_actual: Vec<(usize, DiffType)>,
        diff_a_mergear: Vec<(usize, DiffType)>,
        archivo_base: String,
    ) -> (String, bool) {
        let mut posibles_conflictos: Vec<ConflictoAtomico> = Vec::new();
        let mut hubo_conflictos = false;
        println!("{:?}", diff_actual);
        println!("{:?}", diff_a_mergear);

        for diff in diff_actual {
            if diff.0 - 1 >= posibles_conflictos.len() {
                posibles_conflictos.push(Vec::new());
            }
            posibles_conflictos[diff.0 - 1].push((diff.1, LadoConflicto::Head));
        }

        for diff in diff_a_mergear {
            if diff.0 - 1 > posibles_conflictos.len() {
                posibles_conflictos.push(Vec::new());
            }
            posibles_conflictos[diff.0 - 1].push((diff.1, LadoConflicto::Entrante));
        }

        let mut contenido_por_regiones: Vec<Region> = Vec::new();
        let lineas_archivo_base = archivo_base.split('\n').collect::<Vec<&str>>();

        for i in 0..posibles_conflictos.len() {
            let posible_conflicto = &posibles_conflictos[i];
            if Self::hay_conflicto(posible_conflicto) {
                hubo_conflictos = true;
                contenido_por_regiones.push(Self::resolver_conflicto(
                    posible_conflicto,
                    lineas_archivo_base.iter().nth(i).unwrap_or(&""),
                ));
            } else {
                if posible_conflicto.len() == 2 {
                    contenido_por_regiones.push(resolver_merge_len_2(
                        posible_conflicto,
                        lineas_archivo_base[i],
                    ));
                } else {
                    for (diff, _) in posible_conflicto {
                        if let DiffType::Added(linea) = diff {
                            contenido_por_regiones.push(Region::Normal(linea.clone()))
                        }
                    }
                }
            }
        }

        let regiones_unificadas = unificar_regiones(contenido_por_regiones);

        let mut resultado = String::new();

        for region in &regiones_unificadas {
            resultado.push_str(&format!("{}\n", region));
        }

        (resultado, hubo_conflictos)
    }

    fn automerge(&self, commit_base: String) -> Result<String, String> {
        println!("Realizando automerge");
        let hash_tree_base = write_tree::conseguir_arbol_from_hash_commit(
            &commit_base,
            String::from(".gir/objects/"),
        );
        let tree_base = Tree::from_hash(hash_tree_base, PathBuf::from("."))?;

        let tree_branch_actual = Self::obtener_arbol_commit_actual(self.branch_actual.clone())?;
        let tree_branch_a_mergear =
            Self::obtener_arbol_commit_actual(self.branch_a_mergear.clone())?;

        let nodos_hoja_base = tree_base.obtener_objetos_hoja();
        let nodos_hoja_branch_actual = tree_branch_actual.obtener_objetos_hoja();
        let nodos_hoja_branch_a_mergear = tree_branch_a_mergear.obtener_objetos_hoja();

        let mut objetos_index: Vec<ObjetoIndex> = Vec::new();
        let mut paths_con_conflictos: Vec<String> = Vec::new();

        for objeto_base in nodos_hoja_base {
            let nombre_objeto = objeto_base.obtener_path();
            let objeto_a_mergear_encontrado = nodos_hoja_branch_a_mergear
                .iter()
                .find(|&nodo| nodo.obtener_path() == nombre_objeto);

            let objeto_actual_encontrado = nodos_hoja_branch_actual
                .iter()
                .find(|&nodo| nodo.obtener_path() == nombre_objeto);

            let (objeto_a_mergear, objeto_actual) =
                match (objeto_a_mergear_encontrado, objeto_actual_encontrado) {
                    (Some(objeto_a_mergear), Some(objeto_actual)) => {
                        (objeto_a_mergear, objeto_actual)
                    }
                    (None, Some(objeto_actual)) => {
                        let objeto = ObjetoIndex {
                            objeto: objeto_actual.clone(),
                            es_eliminado: false,
                            merge: false,
                        };

                        objetos_index.push(objeto);
                        continue;
                    }

                    _ => continue,
                };

            let diff_a_mergear = Self::obtener_diffs_entre_dos_objetos(
                objeto_base.obtener_hash(),
                objeto_a_mergear.obtener_hash(),
            )?;

            let diff_actual = Self::obtener_diffs_entre_dos_objetos(
                objeto_base.obtener_hash(),
                objeto_actual.obtener_hash(),
            )?;

            let contenido_base =
                descomprimir_objeto(objeto_base.obtener_hash(), String::from(".gir/objects/"))?;

            let (resultado, hubo_conflictos) =
                Self::mergear_archivos(diff_actual, diff_a_mergear, contenido_base);

            escribir_bytes(objeto_base.obtener_path(), resultado)?;
            if hubo_conflictos {
                paths_con_conflictos.push(format!("{}\n", objeto_base.obtener_path().display()));
            }

            let objeto = ObjetoIndex {
                objeto: objeto_base,
                es_eliminado: false,
                merge: hubo_conflictos,
            };

            objetos_index.push(objeto);
        }

        escribir_index(self.logger.clone(), &objetos_index)?;
        self.escribir_merge_head()?;
        self.escribir_mensaje_merge()?;

        if paths_con_conflictos.len() > 0 {
            Ok(format!(
                "Se encontraron conflictos en los siguientes archivos: \n{:#?}",
                paths_con_conflictos
            ))
        } else {
            let commit = Commit::from_merge(self.logger.clone())?;
            commit.ejecutar()?;
            Ok("Merge completado".to_string())
        }
    }

    pub fn fast_forward(&self) -> Result<String, String> {
        let commit_banch_a_mergear = leer_a_string(Path::new(&format!(
            ".gir/refs/heads/{}",
            self.branch_a_mergear
        )))?;

        io::escribir_bytes(
            Path::new(&format!(".gir/refs/heads/{}", self.branch_actual)),
            commit_banch_a_mergear,
        )?;

        let tree_branch_a_mergear =
            Self::obtener_arbol_commit_actual(self.branch_a_mergear.clone())?;

        tree_branch_a_mergear.escribir_en_directorio()?;
        Ok("Merge con fast-forward completado".to_string())
    }

    pub fn hay_archivos_sin_mergear() -> Result<bool, String> {
        let ruta_index = Path::new(".gir/index");
        if !ruta_index.exists() {
            return Ok(false);
        }
        let contenido_index = leer_index()?;
        if contenido_index.is_empty() {
            return Ok(false);
        }
        Ok(contenido_index.iter().all(|objeto| !objeto.merge))
    }

    pub fn hay_merge_en_curso() -> Result<bool, String> {
        let path = Path::new(".gir/MERGE_HEAD");
        if !path.exists() {
            return Ok(false);
        }

        let merge = leer_a_string(".gir/MERGE_HEAD")?;

        Ok(!merge.is_empty())
    }

    pub fn obtener_commit_de_branch(branch: &String) -> Result<String, String> {
        let ruta = format!(".gir/refs/heads/{}", branch);
        let padre_commit = leer_a_string(path::Path::new(&ruta))?;
        Ok(padre_commit)
    }

    fn escribir_merge_head(&self) -> Result<(), String> {
        let ruta_merge = Path::new(".gir/MERGE_HEAD");
        let commit = Self::obtener_commit_de_branch(&self.branch_a_mergear)?;
        escribir_bytes(ruta_merge, commit)?;
        Ok(())
    }

    fn escribir_mensaje_merge(&self) -> Result<(), String> {
        let ruta_merge_msg = Path::new(".gir/COMMIT_EDITMSG");
        escribir_bytes(
            ruta_merge_msg,
            format!(
                "Mergear rama \"{}\" en  \"{}\"",
                self.branch_a_mergear, self.branch_actual
            ),
        )?;
        Ok(())
    }

    pub fn limpiar_merge_post_commit() -> Result<(), String> {
        let ruta_merge = Path::new(".gir/MERGE_HEAD");
        if ruta_merge.exists() {
            rm_directorio(ruta_merge)?;
        }
        Ok(())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando merge".to_string());

        if Self::hay_merge_en_curso()? {
            return Err("Ya hay un merge en curso".to_string());
        }

        let commit_base = self.obtener_commit_base_entre_dos_branches()?;
        let commit_actual = Commit::obtener_hash_del_padre_del_commit()?;

        let mensaje = if commit_base == commit_actual {
            self.logger.log("Haciendo fast-forward".to_string());
            self.fast_forward()
        } else {
            self.logger.log("Realizando auto-merge".to_string());
            self.automerge(commit_base)
        }?;

        self.escribir_merge_head()?;
        Ok(mensaje)
    }
}

// #[cfg(test)]
// mod test {
//     use crate::tipos_de_dato::comandos::hash_object::HashObject;

//     use super::*;

//     #[test]
//     fn test_computar_lcs_grid() {
//         let mut args = vec!["-w".to_string(), "aaaaa.txt".to_string()];
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/hash_object_test01")).unwrap());
//         let hash_object = HashObject::from(&mut args, logger.clone()).unwrap();
//         let hash_a = hash_object.ejecutar().unwrap();

//         let mut args = vec!["-w".to_string(), "bbbbb.txt".to_string()];
//         let hash_object = HashObject::from(&mut args, logger).unwrap();
//         let hash_b = hash_object.ejecutar().unwrap();

//         let diff =
//             Merge::obtener_diffs_entre_dos_objetos(hash_a.to_string(), hash_b.to_string()).unwrap();
//         println!("{:?}", diff);
//         assert_eq!("aaa", "b");
//     }

//     #[test]
//     fn test_merge_entre_files_segun_base() {
//         let mut args = vec!["-w".to_string(), "aaaaa.txt".to_string()];
//         let logger = Rc::new(Logger::new(PathBuf::from("tmp/hash_object_test01")).unwrap());
//         let hash_object = HashObject::from(&mut args, logger.clone()).unwrap();
//         let hash_a = hash_object.ejecutar().unwrap();

//         let mut args = vec!["-w".to_string(), "bbbbb.txt".to_string()];
//         let hash_object = HashObject::from(&mut args, logger.clone()).unwrap();
//         let hash_b = hash_object.ejecutar().unwrap();

//         let mut args = vec!["-w".to_string(), "ccccc.txt".to_string()];
//         let hash_object = HashObject::from(&mut args, logger).unwrap();
//         let hash_c = hash_object.ejecutar().unwrap();
//         let contenido_base = leer_a_string("ccccc.txt").unwrap();

//         let diff_a_c =
//             Merge::obtener_diffs_entre_dos_objetos(hash_c.to_string(), hash_a.to_string()).unwrap();
//         let diff_b_c =
//             Merge::obtener_diffs_entre_dos_objetos(hash_c.to_string(), hash_b.to_string()).unwrap();
//         let (contenido_final, conflictos) =
//             Merge::mergear_archivos(diff_a_c, diff_b_c, contenido_base);
//         println!("{:?}", contenido_final);
//         assert_eq!(contenido_final, "hola\njuampi\nronaldo\n");
//     }
// }
