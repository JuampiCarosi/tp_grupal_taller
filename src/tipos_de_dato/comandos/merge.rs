use std::{rc::Rc, path::{self, PathBuf}};

use sha1::digest::typenum::Diff;

use crate::{tipos_de_dato::{logger::Logger, objetos::tree::Tree}, io::{leer_a_string, self}, utilidades_de_compresion, utilidades_index::leer_index};

use super::{commit::Commit, log::Log, write_tree::{self, conseguir_arbol_from_hash_commit}, cat_file};

pub struct Merge {
    pub logger: Rc<Logger>,
    pub branch_actual: String,
    pub branch_a_mergear: String,
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
        };
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

    //en vez de devolver el grid, construir el result aca.
    fn obtener_diffs_entre_dos_objetos(hash_objeto1: String, hash_objeto2: String) -> Result<Vec<(usize, DiffGrid)>, String> {
        let (_, contenido1) = cat_file::obtener_contenido_objeto(hash_objeto1)?;
        let (_, contenido2) = cat_file::obtener_contenido_objeto(hash_objeto2)?;
        let mut i: usize= 0;
        let lineas_contenido2: Vec<&str> = contenido2.split('\n').collect();
        let j = lineas_contenido2.len();
        let mut diferencias: Vec<(usize, DiffGrid)> = Vec::new();
        for linea in contenido1.split('\n') {
            if i < j-1 {
                let diff = Self::computar_lcs_grid(linea, &lineas_contenido2[i]);
                diferencias.push((i, diff));
                i += 1;
            } else {
                let diff = Self::computar_lcs_grid(linea, "");
                diferencias.push((i, diff));
            }

        }
        Ok(diferencias)
    }

    fn computar_lcs_grid(x: &str, y: &str) -> DiffGrid {
        let m = x.len();
        let n = y.len();
    
        let mut c = vec![vec![0; n + 1]; m + 1];
    
        for i in 0..=m {
            c[i][0] = 0;
        }
    
        for j in 0..=n {
            c[0][j] = 0;
        }
    
        for i in 0..m {
            for j in 0..n {
                if x.chars().nth(i).unwrap() == y.chars().nth(j).unwrap() {
                    c[i + 1][j + 1] = c[i][j] + 1;
                } else {
                    c[i + 1][j + 1] = c[i + 1][j].max(c[i][j + 1]);
                }
            }
        }
        c
    }
    
    fn print_diff(c: &Vec<Vec<usize>>, x: &str, y: &str, i: usize, j: usize) {
        if i > 0 && j > 0 && x.chars().nth(i - 1).unwrap() == y.chars().nth(j - 1).unwrap() {
            Self::print_diff(c, x, y, i - 1, j - 1);
            println!("  {}", x.chars().nth(i - 1).unwrap());
        } else if j > 0 && (i == 0 || c[i][j - 1] >= c[i - 1][j]) {
            Self::print_diff(c, x, y, i, j - 1);
            println!("> {}", y.chars().nth(j - 1).unwrap());
        } else if i > 0 && (j == 0 || c[i][j - 1] < c[i - 1][j]) {
            Self::print_diff(c, x, y, i - 1, j);
            println!("< {}", x.chars().nth(i - 1).unwrap());
        }
    }

    pub fn ejecutar(&self) -> Result<String, String> {
        self.logger.log("Ejecutando comando merge".to_string());

        let commit_base = self.obtener_commit_base_entre_dos_branches()?;
        let hash_tree_base = write_tree::conseguir_arbol_from_hash_commit(commit_base.clone());
        let tree_base = Tree::from_hash(hash_tree_base, PathBuf::from("."))?;

        let tree_branch_actual = Self::obtener_arbol_commit_actual(self.branch_actual.clone())?;
        let tree_branch_a_mergear = Self::obtener_arbol_commit_actual(self.branch_a_mergear.clone())?;

        // for objeto in tree_base.obtener_objetos_hoja() {
        //     let diff_branch_actual = Self::compute_lcs_grid();
        // }




        self.logger.log("Comando merge ejecutado correctamente".to_string());
        Ok("".to_string())
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_computar_lcs_grid() {
        let x = "994879a4fef366ec15afa83056d67fb0332048d3".to_string();
        let y = "40f40dae766151c6e172ae7d42c4bbc08480481a".to_string();
        let (_, contenido1) = cat_file::obtener_contenido_objeto(x.clone()).unwrap();
        let (_, contenido2) = cat_file::obtener_contenido_objeto(y.clone()).unwrap();
        let diff = Merge::obtener_diffs_entre_dos_objetos(x.to_string(), y.to_string()).unwrap();
        Merge::print_diff(&diff, &contenido1, &contenido2, contenido1.len(), contenido2.len());
        assert_eq!("aaa", "b");
    }
}