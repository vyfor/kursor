use kursor_core::runtime::Runtime;

pub fn next(runtime: &mut Runtime) -> bool {
    towards(runtime, 1)
}

pub fn previous(runtime: &mut Runtime) -> bool {
    towards(runtime, -1)
}

fn towards(runtime: &mut Runtime, direction: isize) -> bool {
    let all = runtime.focusables();
    if all.is_empty() {
        return false;
    }

    let current = runtime.focus();
    let boundary = current.and_then(|id| {
        runtime.path_to_root(id).into_iter().find(|node| {
            runtime.focus_of(*node).is_some_and(|config| config.trap)
        })
    });
    let nodes: Vec<_> = match boundary {
        Some(boundary) => runtime
            .subtree(boundary)
            .into_iter()
            .filter(|node| all.contains(node))
            .collect(),
        None => all,
    };
    if nodes.is_empty() {
        return false;
    }

    let index = current
        .and_then(|id| nodes.iter().position(|candidate| *candidate == id))
        .map_or(if direction > 0 { 0 } else { nodes.len() - 1 }, |index| {
            let next = index as isize + direction;
            next.rem_euclid(nodes.len() as isize) as usize
        });

    runtime.set_focus(Some(nodes[index]));

    true
}
