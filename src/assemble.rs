use std::collections::HashMap;

use crate::parser::{parse, RMLNode};

/// The assemble procedure is the main entry to the assembler.
pub fn assemble(
    controller_text: &str,
) -> Result<(Vec<RMLNode>, HashMap<String, Vec<RMLNode>>), String> {
    let mut labels: HashMap<String, Vec<RMLNode>> = HashMap::new();
    let mut insts: Vec<RMLNode> = vec![];
    let parse_result = parse(controller_text).unwrap();
    let mut current_label = String::from("");
    let mut leading_label: Option<String> = None;
    let mut first_inst = true;

    // Split the parse result into instructions set and label-insts map.
    for node in parse_result.into_iter() {
        match node {
            RMLNode::Symbol(label) => {
                if labels.contains_key(&label) {
                    return Err(format!("[ASSEMBLE] Duplicated label: {}", label));
                }
                if first_inst {
                    leading_label = Some(label.clone());
                }
                first_inst = false;
                current_label = label.clone();
                labels.insert(label, vec![]);
            }
            inst => {
                insts.push(inst.clone());
                if let Some(v) = labels.get_mut(&current_label) {
                    v.push(inst);
                } else {
                    return Err(format!("[ASSEMBLE] Missed label: {}", current_label));
                }
            }
        }
    }
    if let Some(label) = leading_label {
        labels.insert(label, insts.clone());
    }
    Ok((insts, labels))
}
