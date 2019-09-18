
use std::sync::{Mutex, Arc};
use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Serialize, Deserialize)]
pub struct TreeNode {
    parent: Option<Box<TreeNode>>,
    children: Vec<Box<TreeNode>>,
    id: i64,
    label: String,
    cost: i64,
    calls: i64
}

impl TreeNode {
    pub fn new(id: i64, label: String, cost: i64, calls: i64) -> TreeNode {
        TreeNode {
            parent: None,
            children: vec![],
            id,
            label,
            cost,
            calls
        }
    }

    pub fn appendChild(&mut self, childNode: TreeNode) {
        self.cost += childNode.cost;
        self.children.push(Box::new(childNode));
    }

}