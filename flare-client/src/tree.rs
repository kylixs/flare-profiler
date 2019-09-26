
use std::sync::{Mutex, Arc};
use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sampler_client::ThreadData;


#[derive(Serialize, Deserialize)]
pub struct TreeNode {
    pub parent: Option<Box<TreeNode>>,
    pub children: Vec<Box<TreeNode>>,
    pub id: i64,
    pub label: String,
    pub calls: i64,
    pub cpu: i64,
    pub duration: i64,
}

impl TreeNode {
    pub fn new(id: i64, label: &str) -> TreeNode {
        TreeNode {
            parent: None,
            children: vec![],
            id,
            label: label.to_string(),
            calls: 0,
            cpu: 0,
            duration: 0
        }
    }

    pub fn append_child<'a>(&'a mut self, childNode: TreeNode) -> &'a Box<TreeNode> {
//        self.cost += childNode.cost;
        let node = Box::new(childNode);
        self.children.push(node);
        self.children.last().as_mut().unwrap()
    }

    pub fn last_child(&self) -> Option<&Box<TreeNode>> {
        self.children.last()
    }

    pub fn merge_or_create_last_child<'a>(node: &'a mut Box<TreeNode>, method_name: &str, self_duration: i64, self_cpu_time: i64, samples: i64) -> &'a mut Box<TreeNode> {
        let mut last_node = node.children.last();
        if last_node.is_none() {
            last_node = Some(node.append_child(TreeNode{
                parent: None,
                children: vec![],
                id: 0,
                label: method_name.to_string(),
                calls: 0,
                cpu: 0,
                duration: 0
            }));
        }
        let mut last_node = last_node.unwrap();
        last_node.duration += self_duration;
        last_node.cpu += self_cpu_time;
        last_node.calls += samples;
        last_node
    }
}
