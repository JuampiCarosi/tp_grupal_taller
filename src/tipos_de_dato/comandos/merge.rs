use std::{
    path::{self, Path, PathBuf},
    rc::Rc,
};

use crate::{
    io::{self, leer_a_string},
    tipos_de_dato::{logger::Logger, objetos::tree::Tree},
    utilidades_de_compresion,
};

use super::{
    cat_file,
    commit::{self, Commit},
    log::Log,
    write_tree::{self, conseguir_arbol_from_hash_commit},
};

pub struct Merge {
    pub logger: Rc<Logger>,
    pub branch_actual: String,
    pub branch_a_mergear: String,
}

enum LadoConflicto {
    Head,
    Entrante,
}
#[derive(Debug, Clone)]

enum DiffType {
    Added(String),
    Removed(String),
    Unchanged(String),
}
type Conflicto = Vec<(DiffType, LadoConflicto)>;

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
        let hash_tree_padre = conseguir_arbol_from_hash_commit(head_commit);
        Ok(Tree::from_hash(hash_tree_padre, PathBuf::from("."))?)
    }

    fn obtener_listas_de_commits(branch: &String) -> Result<Vec<String>, String> {
        let ruta = format!(".gir/refs/heads/{}", branch);
        let mut ultimo_commit = leer_a_string(path::Path::new(&ruta))?;

        if ultimo_commit.is_empty() {
            return Ok(Vec::new());
        }
        let mut historial_commits: Vec<String> = Vec::new();
        loop {
            let contenido = utilidades_de_compresion::descomprimir_objeto(ultimo_commit.clone())?;
            let siguiente_padre = Log::conseguir_padre_desde_contenido_commit(&contenido);
            historial_commits.push(ultimo_commit.clone());
            if siguiente_padre.is_empty() {
                break;
            }
            ultimo_commit = siguiente_padre.to_string();
        }
        Ok(historial_commits)
    }

    fn obtener_commit_base_entre_dos_branches(&self) -> Result<String, String> {
        let commits_branch_actual = Self::obtener_listas_de_commits(&self.branch_actual)?;
        let commits_branch_a_mergear = Self::obtener_listas_de_commits(&self.branch_a_mergear)?;

        for commit_actual in commits_branch_actual {
            for commit_branch_merge in commits_branch_a_mergear.clone() {
                if commit_actual == commit_branch_merge {
                    return Ok(commit_actual);
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
    //en texto 1 debe ir la base
    fn obtener_diff(texto1: Vec<&str>, texto2: Vec<&str>) -> Vec<(usize, DiffType)> {
        let diff_grid = Self::computar_lcs_grid(&texto1, &texto2);
        let mut i = texto1.len();
        let mut j = texto2.len();
        let mut resultado_diff: Vec<(usize, DiffType)> = Vec::new();

        while i != 0 || j != 0 {
            if i == 0 {
                resultado_diff.push((i, DiffType::Added(texto2[j - 1].trim().to_string())));
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
                i -= 1;
            }
        }
        resultado_diff.reverse();
        resultado_diff
    }

    fn no_hay_conflicto(conflicto: &Vec<(DiffType, LadoConflicto)>) -> bool {
        for diff in conflicto {
            if let DiffType::Unchanged(_) = diff.0 {
                continue;
            } else {
                return false;
            }
        }

        return true;
    }

    fn mergear_archivos(
        diff_actual: Vec<(usize, DiffType)>,
        diff_a_mergear: Vec<(usize, DiffType)>,
    ) -> String {
        println!("{:?}", diff_actual);
        println!("{:?}", diff_a_mergear);

        let mut conflictos: Vec<Conflicto> = Vec::new();

        for diff in diff_actual {
            if diff.0 - 1 >= conflictos.len() {
                conflictos.push(Vec::new());
            }
            conflictos[diff.0 - 1].push((diff.1, LadoConflicto::Head));
        }

        for diff in diff_a_mergear {
            if diff.0 - 1 > conflictos.len() {
                conflictos.push(Vec::new());
            }
            conflictos[diff.0 - 1].push((diff.1, LadoConflicto::Entrante));
        }

        let mut contenido_final = String::new();

        for conflicto in conflictos {}

        contenido_final
    }

    fn automerge(&self, commit_base: String) -> Result<(), String> {
        let hash_tree_base = write_tree::conseguir_arbol_from_hash_commit(commit_base.clone());
        let tree_base = Tree::from_hash(hash_tree_base, PathBuf::from("."))?;

        let tree_branch_actual = Self::obtener_arbol_commit_actual(self.branch_actual.clone())?;
        let tree_branch_a_mergear =
            Self::obtener_arbol_commit_actual(self.branch_a_mergear.clone())?;

        let nodos_hoja_base = tree_base.obtener_objetos_hoja();
        let nodos_hoja_branch_actual = tree_branch_actual.obtener_objetos_hoja();
        let nodos_hoja_branch_a_mergear = tree_branch_a_mergear.obtener_objetos_hoja();

        for objeto in nodos_hoja_base {
            let nombre_objeto = objeto.obtener_path();
            let objeto_a_mergear = nodos_hoja_branch_a_mergear
                .iter()
                .find(|&nodo| nodo.obtener_path() == nombre_objeto);
            if objeto_a_mergear.is_none() {
                continue;
            }
            let objeto_a_mergear = objeto_a_mergear.unwrap();
            let diff_a_mergear = Self::obtener_diffs_entre_dos_objetos(
                objeto.obtener_hash(),
                objeto_a_mergear.obtener_hash(),
            )?;

            let objeto_actual = nodos_hoja_branch_actual
                .iter()
                .find(|&nodo| nodo.obtener_path() == nombre_objeto);
            if objeto_actual.is_none() {
                continue;
            }
            let objeto_actual = objeto_actual.unwrap();
            let diff_actual = Self::obtener_diffs_entre_dos_objetos(
                objeto.obtener_hash(),
                objeto_actual.obtener_hash(),
            )?;
        }
        Ok(())
    }

    pub fn fast_forward(&self) -> Result<(), String> {
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

        Ok(())
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando merge".to_string());

        let commit_base = self.obtener_commit_base_entre_dos_branches()?;
        let commit_actual = Commit::obtener_hash_del_padre_del_commit()?;

        if commit_base == commit_actual {
            self.fast_forward()?
        } else {
            self.automerge(commit_base)?
        };

        self.logger
            .log("Comando merge ejecutado correctamente".to_string());
        Ok("Merge completado".to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::tipos_de_dato::comandos::hash_object::HashObject;

    use super::*;

    #[test]
    fn test_computar_lcs_grid() {
        let mut args = vec!["-w".to_string(), "aaaaa.txt".to_string()];
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/hash_object_test01")).unwrap());
        let hash_object = HashObject::from(&mut args, logger.clone()).unwrap();
        let hash_a = hash_object.ejecutar().unwrap();

        let mut args = vec!["-w".to_string(), "bbbbb.txt".to_string()];
        let hash_object = HashObject::from(&mut args, logger).unwrap();
        let hash_b = hash_object.ejecutar().unwrap();

        let diff =
            Merge::obtener_diffs_entre_dos_objetos(hash_a.to_string(), hash_b.to_string()).unwrap();
        println!("{:?}", diff);
        assert_eq!("aaa", "b");
    }

    #[test]
    fn test_merge_entre_files_segun_base() {
        let mut args = vec!["-w".to_string(), "aaaaa.txt".to_string()];
        let logger = Rc::new(Logger::new(PathBuf::from("tmp/hash_object_test01")).unwrap());
        let hash_object = HashObject::from(&mut args, logger.clone()).unwrap();
        let hash_a = hash_object.ejecutar().unwrap();

        let mut args = vec!["-w".to_string(), "bbbbb.txt".to_string()];
        let hash_object = HashObject::from(&mut args, logger.clone()).unwrap();
        let hash_b = hash_object.ejecutar().unwrap();

        let mut args = vec!["-w".to_string(), "ccccc.txt".to_string()];
        let hash_object = HashObject::from(&mut args, logger).unwrap();
        let hash_c = hash_object.ejecutar().unwrap();

        let diff_a_c =
            Merge::obtener_diffs_entre_dos_objetos(hash_c.to_string(), hash_a.to_string()).unwrap();
        let diff_b_c =
            Merge::obtener_diffs_entre_dos_objetos(hash_c.to_string(), hash_b.to_string()).unwrap();
        let contenido_final = Merge::mergear_archivos(diff_a_c, diff_b_c);
        println!("{:?}", contenido_final);
        assert_eq!(contenido_final, "hola\nmateo\njuampi\n");
    }
}
