
use std::sync::{Mutex, Arc};
use std::cmp::min;
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Serialize, Deserialize)]
pub struct TreeNode {
    pub parent: Option<Box<TreeNode>>,
    pub children: Vec<Box<TreeNode>>,
    pub id: i64,
    pub label: String,
//    cost: i64,
//    calls: i64
}

impl TreeNode {
    pub fn new(id: i64, label: String) -> TreeNode {
        TreeNode {
            parent: None,
            children: vec![],
            id,
            label,
        }
    }

    pub fn appendChild(&mut self, childNode: TreeNode) {
//        self.cost += childNode.cost;
        self.children.push(Box::new(childNode));
    }

}