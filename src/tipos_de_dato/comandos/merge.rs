use std::{
    path::{self, PathBuf},
    rc::Rc,
};

use crate::{
    io::{self, leer_a_string},
    tipos_de_dato::{logger::Logger, objetos::tree::Tree},
    utilidades_de_compresion,
};

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
enum DiffType {
    Added(String),
    Removed(String),
    Unchanged(String),
}
#[derive(Debug, Clone)]
enum CompleteDiffType {
    Added(String),
    Removed(String),
    Unchanged(String),
    Conflict(DiffType, DiffType),
}

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
            if siguiente_padre.is_empty() {
                break;
            }
            historial_commits.push(ultimo_commit.clone());
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
    ) -> Result<Vec<DiffType>, String> {
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
    fn obtener_diff(texto1: Vec<&str>, texto2: Vec<&str>) -> Vec<DiffType> {
        let diff_grid = Self::computar_lcs_grid(&texto1, &texto2);
        let mut i = texto1.len();
        let mut j = texto2.len();
        let mut resultado_diff: Vec<DiffType> = Vec::new();

        while i != 0 || j != 0 {
            if i == 0 {
                resultado_diff.push(DiffType::Added(texto2[j - 1].trim().to_string()));
                j -= 1;
            } else if j == 0 {
                resultado_diff.push(DiffType::Removed(texto1[i - 1].trim().to_string()));
                i -= 1;
            } else if texto1[i - 1] == texto2[j - 1] {
                resultado_diff.push(DiffType::Unchanged(texto1[i - 1].trim().to_string()));
                i -= 1;
                j -= 1;
            } else if diff_grid[i - 1][j] <= diff_grid[i][j - 1] {
                resultado_diff.push(DiffType::Added(texto2[j - 1].trim().to_string()));
                j -= 1;
            } else {
                resultado_diff.push(DiffType::Removed(texto1[i - 1].trim().to_string()));
                i -= 1;
            }
        }
        resultado_diff.reverse();
        resultado_diff
    }

    fn mergear_archivos(diff_actual: Vec<DiffType>, diff_a_mergear: Vec<DiffType>) -> String {
        println!("{:?}", diff_actual);
        println!("{:?}", diff_a_mergear);
        let mut i = 0;
        let mut j = 0;
        let mut conflictos: Vec<(usize, usize)> = Vec::new();
        let mut contenido_final = String::new();
        let mut diff_total: Vec<CompleteDiffType> = Vec::new();
        while i < diff_actual.len() && j < diff_a_mergear.len() {
            match (diff_actual[i].clone(), diff_a_mergear[j].clone()) {
                (DiffType::Unchanged(linea1), DiffType::Unchanged(_)) => {
                    i += 1;
                    j += 1;
                    contenido_final.push_str(&format!("{}\n", &linea1));
                    diff_total.push(CompleteDiffType::Unchanged(linea1));
                }
                (DiffType::Unchanged(_), DiffType::Added(linea2)) => {
                    j += 1;
                    contenido_final.push_str(&format!("{}\n", &linea2));
                    diff_total.push(CompleteDiffType::Added(linea2));
                }
                (DiffType::Unchanged(_), DiffType::Removed(linea2)) => {
                    i += 1;
                    j += 1;
                    diff_total.push(CompleteDiffType::Removed(linea2));
                }
                (DiffType::Added(linea1), DiffType::Unchanged(_)) => {
                    i += 1;
                    contenido_final.push_str(&format!("{}\n", &linea1));
                    diff_total.push(CompleteDiffType::Added(linea1));
                }
                (DiffType::Removed(linea1), DiffType::Unchanged(_)) => {
                    i += 1;
                    j += 1;
                    diff_total.push(CompleteDiffType::Removed(linea1));
                }
                (DiffType::Added(linea1), DiffType::Added(linea2)) => {
                    i += 1;
                    j += 1;
                    if linea1 != linea2 {
                        conflictos.push((i, j));
                        contenido_final.push_str("<<<<<<<<<<<<<\n");
                        contenido_final.push_str(&format!("{}\n", &linea1));
                        contenido_final.push_str("===============\n");
                        contenido_final.push_str(&format!("{}\n", &linea2));
                        contenido_final.push_str(">>>>>>>>>>>>>\n");
                        diff_total.push(CompleteDiffType::Conflict(
                            DiffType::Added(linea1),
                            DiffType::Added(linea2),
                        ))
                    } else {
                        contenido_final.push_str(format!("{}\n", &linea1).as_str());
                        diff_total.push(CompleteDiffType::Added(linea1));
                    }
                }
                (DiffType::Added(linea1), DiffType::Removed(linea2)) => {
                    i += 1;
                    j += 1;
                    conflictos.push((i, j));
                    contenido_final.push_str("<<<<<<<<<<<<<\n");
                    contenido_final.push_str(&format!("{}\n", &linea1));
                    contenido_final.push_str("===============\n");
                    contenido_final.push_str(">>>>>>>>>>>>>\n");
                    diff_total.push(CompleteDiffType::Conflict(
                        DiffType::Added(linea1),
                        DiffType::Removed(linea2),
                    ))
                }
                (DiffType::Removed(linea1), DiffType::Added(linea2)) => {
                    j += 1;
                    i += 1;
                    conflictos.push((i, j));
                    contenido_final.push_str("<<<<<<<<<<<<<\n");
                    contenido_final.push_str("===============\n");
                    contenido_final.push_str(&format!("{}\n", &linea2));
                    contenido_final.push_str(">>>>>>>>>>>>>\n");
                    diff_total.push(CompleteDiffType::Conflict(
                        DiffType::Removed(linea1),
                        DiffType::Added(linea2),
                    ))
                }
                (DiffType::Removed(linea1), DiffType::Removed(linea2)) => {
                    i += 1;
                    j += 1;
                    if linea1 != linea2 {
                        conflictos.push((i, j));
                        contenido_final.push_str("<<<<<<<<<<<<<\n");
                        contenido_final.push_str(&format!("{}\n", &linea1));
                        contenido_final.push_str("===============\n");
                        contenido_final.push_str(&format!("{}\n", &linea2));
                        contenido_final.push_str(">>>>>>>>>>>>>\n");
                        diff_total.push(CompleteDiffType::Conflict(
                            DiffType::Removed(linea1),
                            DiffType::Removed(linea2),
                        ))
                    }
                }
            }
        }
        while i < diff_actual.len() {
            match diff_actual[i].clone() {
                DiffType::Unchanged(linea1) => {
                    i += 1;
                    contenido_final.push_str(&format!("{}\n", &linea1));
                    diff_total.push(CompleteDiffType::Unchanged(linea1));
                }
                DiffType::Added(linea1) => {
                    i += 1;
                    contenido_final.push_str(&format!("{}\n", &linea1));
                    diff_total.push(CompleteDiffType::Added(linea1));
                }
                DiffType::Removed(linea1) => {
                    i += 1;
                    diff_total.push(CompleteDiffType::Removed(linea1));
                }
            }
        }
        while j < diff_a_mergear.len() {
            match diff_a_mergear[j].clone() {
                DiffType::Unchanged(linea2) => {
                    j += 1;
                    contenido_final.push_str(&format!("{}\n", &linea2));
                    diff_total.push(CompleteDiffType::Unchanged(linea2));
                }
                DiffType::Added(linea2) => {
                    j += 1;
                    contenido_final.push_str(&format!("{}\n", &linea2));
                    diff_total.push(CompleteDiffType::Added(linea2));
                }
                DiffType::Removed(linea2) => {
                    j += 1;
                    diff_total.push(CompleteDiffType::Removed(linea2));
                }
            }
        }
        println!("{:?}", diff_total);
        contenido_final
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando merge".to_string());

        let commit_base = self.obtener_commit_base_entre_dos_branches()?;
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
        self.logger
            .log("Comando merge ejecutado correctamente".to_string());
        Ok("".to_string())
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
