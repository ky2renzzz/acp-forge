//! Skill composition — chain skills into an ordered pipeline.
//!
//! Resolves `depends_on` relationships via topological sort and
//! detects cycles.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::{Error, Result};
use crate::registry::Registry;
use crate::skill::Skill;

/// A resolved, ordered pipeline of skills.
#[derive(Debug)]
pub struct Pipeline {
    /// Skills in execution order (dependencies first).
    pub steps: Vec<Skill>,
}

/// Resolve a set of skill names into an ordered pipeline.
///
/// Uses Kahn's algorithm for topological sort.
pub fn resolve(registry: &Registry, root_names: &[&str]) -> Result<Pipeline> {
    // Collect all transitively required skills.
    let mut needed: HashMap<String, Skill> = HashMap::new();
    let mut queue: VecDeque<String> = root_names.iter().map(|s| s.to_string()).collect();

    while let Some(name) = queue.pop_front() {
        if needed.contains_key(&name) {
            continue;
        }
        let skill = registry.latest(&name)?.clone();
        for dep in &skill.depends_on {
            queue.push_back(dep.clone());
        }
        needed.insert(name, skill);
    }

    // Build adjacency: skill -> set of skills it depends on.
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();

    for (name, skill) in &needed {
        in_degree.entry(name.as_str()).or_insert(0);
        for dep in &skill.depends_on {
            *in_degree.entry(name.as_str()).or_insert(0) += 1;
            dependents
                .entry(dep.as_str())
                .or_default()
                .push(name.as_str());
        }
    }

    // Kahn's algorithm.
    let mut ready: VecDeque<&str> = in_degree
        .iter()
        .filter(|&(_, &deg)| deg == 0)
        .map(|(&name, _)| name)
        .collect();

    let mut order: Vec<Skill> = Vec::new();
    let mut visited = HashSet::new();

    while let Some(name) = ready.pop_front() {
        if !visited.insert(name) {
            continue;
        }
        if let Some(skill) = needed.get(name) {
            order.push(skill.clone());
        }
        if let Some(deps) = dependents.get(name) {
            for &dep in deps {
                let deg = in_degree.get_mut(dep).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    ready.push_back(dep);
                }
            }
        }
    }

    if order.len() != needed.len() {
        let remaining: Vec<_> = needed
            .keys()
            .filter(|k| !visited.contains(k.as_str()))
            .cloned()
            .collect();
        return Err(Error::CompositionCycle {
            chain: remaining.join(" -> "),
        });
    }

    Ok(Pipeline { steps: order })
}
