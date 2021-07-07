use std::collections::HashMap;

use crate::parser::{parse, RMLNode};

/// The assemble procedure is the main entry to the assembler.
pub fn assemble(
    controller_text: &str,
) -> Result<(Vec<RMLNode>, HashMap<String, Vec<RMLNode>>), String> {
    let mut labels: HashMap<String, Vec<RMLNode>> = HashMap::new();
    let mut insts: Vec<RMLNode> = vec![];
    let parse_result = parse(controller_text).unwrap();

    // Split the parse result into instructions set and label-insts map.
    for node in parse_result.into_iter() {
        match node {
            RMLNode::Symbol(label) => {
                if labels.contains_key(&label) {
                    return Err(format!("[ASSEMBLE] Duplicated label: {}", label));
                }
                labels.insert(label, vec![]);
            }
            inst => {
                for (_, val) in labels.iter_mut() {
                    val.push(inst.clone());
                }
                insts.push(inst);
            }
        }
    }
    Ok((insts, labels))
}
