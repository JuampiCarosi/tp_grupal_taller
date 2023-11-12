use super::{region::Region, ConflictoAtomico, DiffType, LadoConflicto};

pub fn conflicto_len_4(conflicto: &ConflictoAtomico) -> Region {
    let mut lado_head = String::new();
    let mut lado_entrante = String::new();

    for (diff, lado) in conflicto {
        if let DiffType::Added(linea) = diff {
            match lado {
                LadoConflicto::Head => lado_head.push_str(&format!("{}\n", linea)),
                LadoConflicto::Entrante => lado_entrante.push_str(&format!("{}\n", linea)),
            };
        }
    }

    Region::Conflicto(lado_head, lado_entrante)
}

pub fn un_lado_conflicto_len_3(conflicto: Vec<&DiffType>, linea_base: &str) -> String {
    let mut lado = String::new();
    if conflicto.len() == 1 {
        match conflicto[0] {
            DiffType::Added(ref linea) => lado.push_str(&format!("{linea_base}\n{linea}\n")),
            DiffType::Unchanged(ref linea) => lado.push_str(&format!("{linea}")),
            _ => {}
        };
    } else {
        for diff in conflicto {
            match diff {
                DiffType::Added(ref linea) => lado.push_str(&format!("{linea}\n")),
                DiffType::Unchanged(ref linea) => lado.push_str(&format!("{linea}\n")),
                _ => {}
            };
        }
    }

    lado
}

pub fn conflicto_len_3(conflicto: &ConflictoAtomico, linea_base: &str) -> Region {
    let head = conflicto
        .iter()
        .filter_map(|(diff, lado)| match lado {
            LadoConflicto::Head => Some(diff),
            _ => None,
        })
        .collect::<Vec<&DiffType>>();

    let lado_head = un_lado_conflicto_len_3(head, linea_base);

    let entrante = conflicto
        .iter()
        .filter_map(|(diff, lado)| match lado {
            LadoConflicto::Entrante => Some(diff),
            _ => None,
        })
        .collect::<Vec<&DiffType>>();

    let lado_entrante = un_lado_conflicto_len_3(entrante, linea_base);

    Region::Conflicto(lado_head, lado_entrante)
}

pub fn resolver_merge_len_2(
    conflicto: &ConflictoAtomico,
    linea_base: &str,
    es_conflicto_obligatorio: bool,
) -> Region {
    match (&conflicto[0].0, &conflicto[1].0) {
        (DiffType::Added(linea_1), DiffType::Added(linea_2)) => {
            if linea_1 != linea_2 || es_conflicto_obligatorio {
                Region::Conflicto(linea_1.clone(), linea_2.clone())
            } else {
                Region::Normal(linea_1.clone())
            }
        }
        (DiffType::Added(linea_1), DiffType::Removed(_)) => {
            Region::Conflicto(format!("{linea_base}\n{linea_1}\n"), "".to_string())
        }
        (DiffType::Added(linea_1), DiffType::Unchanged(linea_2)) => {
            if es_conflicto_obligatorio {
                Region::Conflicto(linea_1.to_owned(), linea_2.to_owned())
            } else {
                Region::Normal(linea_1.clone())
            }
        }
        (DiffType::Removed(_), DiffType::Added(linea_2)) => {
            Region::Conflicto("".to_string(), format!("{linea_base}\n{linea_2}\n"))
        }
        (DiffType::Unchanged(linea_1), DiffType::Added(linea_2)) => {
            if es_conflicto_obligatorio {
                Region::Conflicto(linea_1.to_owned(), linea_2.to_owned())
            } else {
                Region::Normal(linea_2.clone())
            }
        }
        (DiffType::Unchanged(linea_1), DiffType::Unchanged(linea_2)) => {
            if es_conflicto_obligatorio {
                Region::Conflicto(linea_1.to_owned(), linea_2.to_owned())
            } else {
                Region::Normal(linea_1.clone())
            }
        }
        (_, _) => Region::Normal("".to_string()),
    }
}
