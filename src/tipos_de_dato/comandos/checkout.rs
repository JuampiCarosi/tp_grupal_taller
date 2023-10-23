use std::rc::Rc;

use crate::tipos_de_dato::{comandos::branch::Branch, logger::Logger, objetos::tree::Tree};

enum Referencia {
    Commit(String),
    Branch(String),
}

pub struct Checkout {
    crear_branch: bool,
    referencia_a_switchear: Referencia,
    logger: Rc<Logger>,
}

fn verificar_si_la_branch_existe(branch_buscada: String) -> Result<bool, String> {
    let binding = Branch::mostrar_ramas()?;
    let branches: Vec<&str> = binding.split('\n').collect();
    for branch in branches {
        if branch == branch_buscada {
            return Ok(true);
        }
    }
    return Ok(false);
}

fn verificar_si_el_commit_existe(hash_commit_buscado: String) -> bool {
    match Tree::obtener_hash_completo(hash_commit_buscado.clone()) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn obtener_referencia(referencia: String) -> Result<Referencia, String>  {
    let es_branch = verificar_si_la_branch_existe(referencia.clone())?;
    let es_commit = verificar_si_el_commit_existe(referencia.clone());
    if es_branch {
        return Ok(Referencia::Branch(referencia));
    } else if es_commit {
        return Ok(Referencia::Commit(referencia));
    } else {
        return Err("Parametro invalido".to_string());
    }
}

impl Checkout {
    pub fn from(args: Vec<String>, logger: Rc<Logger>) -> Result<Checkout, String> {
        if args.len() > 2 {
            return Err("Demasiados argumentos".to_string());
        }

        if args.len() == 1 {
            let referencia = obtener_referencia(args[0].clone())?;
            return Ok(Checkout {
                crear_branch: false,
                referencia_a_switchear: referencia,
                logger,
            });
        }

        match (args[1].as_str(), args[2].clone()) {
            ("-b", branch) => {
                return Ok(Checkout {
                    crear_branch: true,
                    referencia_a_switchear: Referencia::Branch(branch),
                    logger,
                })
            }
            // (commit, nombre_archivo) => {
            //     return Ok(Checkout {
            //         crear_branch,
            //         referencia_a_switchear: Referencia::Commit(commit),
            //         logger,
            //     })
            // }
            _ => Err("Argumentos invalidos".to_string()),
        }

    }



    pub fn ejecutar(&self) -> Result<String, String> {
        if self.crear_branch {
            let nombre_branch = match self.referencia_a_switchear {
                Referencia::Branch(ref nombre) => nombre.clone(),
                _ => return Err("Argumentos invalidos".to_string()),
            };
            let mut branch = Branch::from(&mut vec![nombre_branch], self.logger.clone())?;
            return branch.ejecutar();
        };
        Ok("".to_string())
    }
}
