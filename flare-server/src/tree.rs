
use std::sync::{Mutex, Arc};
use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sample::ThreadData;


#[derive(Serialize, Deserialize)]
pub struct TreeNode {
    pub parent: Option<Box<TreeNode>>,
    pub children: Vec<Box<TreeNode>>,
    pub id: i64,
    pub label: String,
    pub calls: i64,
    pub cpu: i64,
    pub duration: i64,
    pub start_time: i64,
    pub depth: i32
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
            duration: 0,
            start_time: 0,
            depth: 0
        }
    }

    pub fn append_child<'a>(&'a mut self, childNode: TreeNode) -> &'a mut Box<TreeNode> {
//        self.cost += childNode.cost;
        let node = Box::new(childNode);
        self.children.push(node);
        self.children.last_mut().unwrap()
    }

    pub fn last_child(&mut self) -> Option<&mut Box<TreeNode>> {
        self.children.last_mut()
    }

    pub fn merge_last_child(&mut self, method_name: &str, self_duration: i64, self_cpu_time: i64, samples: i64) -> bool {
        let mut last_node = self.last_child();
        if last_node.is_some() {
            let last_node = last_node.unwrap();
            if last_node.label == method_name {
                last_node.duration += self_duration;
                last_node.cpu += self_cpu_time;
                last_node.calls += samples;
                return true;
            }
        }
        return false;
    }
}
