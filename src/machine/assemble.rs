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

    // Split the parse result into instructions set and label-insts map.
    for node in parse_result.into_iter() {
        match node {
            RMLNode::Symbol(label) => {
                if labels.contains_key(&label) {
                    return Err(format!("[ASSEMBLE] Duplicated label: {}", label));
                }
                if !insts.is_empty() {
                    // Instructions start with a control label are edge case.
                    // If without checking empty, the result will lost the execute order.
                    current_label = label;
                }
                labels.insert(label, vec![]);
            }
            inst => {
                if current_label == "" {
                    insts.push(inst);
                } else {
                    if let Some(&mut v) = labels.get_mut(&current_label) {
                        v.push(inst);
                    } else {
                        return Err(format!("[ASSEMBLE] Missed label: {}", current_label));
                    }
                }
            }
        }
    }
    Ok((insts, labels))
}
