
use std::collections::HashMap;
use std::rc::*;
use std::borrow::Cow;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::Arc;
use time::Duration;
use serde_json::{json, Value};

use log::{debug, info, warn};

use std::collections::hash_map::IterMut;
use tree;

type JavaLong = i64;
type JavaMethod = i64;

static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

fn get_next_nodeid() {
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
}

// assume thread safe, get lock outside
pub struct TreeArena {
    thread_trees: HashMap<JavaLong, CallStackTree>,
//    lock: RwLock<u32>
}

impl TreeArena {
    pub fn new() -> TreeArena {
        TreeArena {
            thread_trees: HashMap::new(),
            //lock: RwLock::new(0)
        }
    }

    pub fn get_all_call_trees(&self) -> &HashMap<JavaLong, CallStackTree>{
        &self.thread_trees
    }

    pub fn get_call_tree(&mut self, thread_id: JavaLong, thread_name: &str) -> &mut CallStackTree {
        self.thread_trees.entry(thread_id).or_insert_with(||{
            CallStackTree::new(thread_id, thread_name)
        })
    }

    pub fn print_all(&self) {
        for (thread_id,thread_data) in self.thread_trees.iter() {
            println!("call tree of thread: [{}]", thread_id);
            println!("{}", thread_data.format_call_tree(false));
        }
    }

    pub fn clear(&mut self) {
        self.thread_trees.clear();
        println!("clear trace data");
    }
}


pub struct CallStackTree {
    nodes: Vec<TreeNode>,
    root_node: NodeId,
    top_call_stack_node: NodeId,
    pub total_duration: i64,
    pub total_cpu: i64,
    pub thread_id: JavaLong
}

impl CallStackTree {

    pub fn new(thread_id: JavaLong, thread_name: &str) -> CallStackTree {
        CallStackTree {
            nodes: vec![TreeNode::newRootNode(thread_name)],
            root_node: NodeId { index: 0 },
            top_call_stack_node: NodeId { index: 0 },
            total_duration: 0,
            total_cpu: 0,
            thread_id
        }
    }

    pub fn reset_top_call_stack_node(&mut self) {
        self.top_call_stack_node = self.root_node;
    }

    pub fn begin_call(&mut self, method_id: &JavaMethod, duration: i64, cpu_time: i64) -> bool {
        //find exist call node
        let topNode = self.get_top_node();
        let child = topNode.find_child(method_id).map(|x|*x).clone();
        match child {
            Some(child_id) => {
                let node = self.get_mut_node(&child_id);
                node.data.call_count += 1;
                node.data.call_duration += duration;
                node.data.call_cpu += cpu_time;
                self.top_call_stack_node = node.data.node_id.clone();
                true
            },
            None => {
                //add new call node
                // Get the next free index
                let next_index = self.nodes.len();

                let topNode = self.get_mut_top_node();
                let mut node = TreeNode::newCallNode(topNode, next_index, method_id);
                node.data.call_count += 1;
                node.data.call_duration += duration;
                node.data.call_cpu += cpu_time;
                self.top_call_stack_node = node.data.node_id.clone();

                // Push the node into the arena
                self.nodes.push(node);
                false
            }
        }
    }

//    pub fn end_call(&mut self, method_id: JavaMethod, call_name: &String, duration: i64) {
//        //let top_node = self.nodes[self.top_call_stack_node.index];
//        let top_node = self.get_mut_top_node();
//        if top_node.data.name == *call_name {
//            top_node.data.call_duration += duration;
//            top_node.data.call_count += 1;
//
//            debug!("end_call: {} {}, call_count:{}", call_name, duration, top_node.data.call_count);
//
//            //pop stack
//            //let parentNode = self.get_node(top_node.parent);
//            //self.top_call_stack_node = top_node.parent.unwrap().clone();
//            match &top_node.parent {
//                Some(nodeid) => {
//                    self.top_call_stack_node = nodeid.clone();
//                },
//                None => {
//                    println!("parent node not found, pop call stack failed, call_name: {}, stack: {}, depth: {}",
//                             call_name, top_node.data.name, top_node.data.depth)
//                }
//            }
//        } else {
//            println!("call name mismatch, pop call stack failed, call_name: {}, top_node:{}, stack:{}, depth: {} ",
//                     call_name, top_node.data.name, top_node.data.name, top_node.data.depth);
//        }
//    }

//    pub fn end_last_call(&mut self, total_duration: i64) {
//        let last_duration = self.total_duration;
//        let top_node = self.get_mut_top_node();
//        //ignore first call duration
//        if(last_duration > 0){
//            top_node.data.call_duration += (total_duration - last_duration);
//        }
//        top_node.data.call_count += 1;
//        self.total_duration = total_duration;
//    }

    //开始合并调用栈，返回本次增量时间 (delta_duration, delta_cpu)
    pub fn start_call_stack(&mut self, total_duration: i64, total_cpu: i64) -> (i64,i64) {
        let last_duration = self.total_duration;
        let last_cpu = self.total_cpu;
        self.total_duration = total_duration;
        self.total_cpu = total_cpu;
        let delta_duration = if last_duration > 0 { total_duration - last_duration }else { 0 };
        let delta_cpu = if last_cpu > 0 {total_cpu - last_cpu} else { 0 };
        (delta_duration, delta_cpu)
    }

    //
    // compact: bool 是否为紧凑模式，即树结点深度使用数字表示。如果为false，则树深度使用多个' '表示
    //
    pub fn format_call_tree(&self, compact: bool) -> String {
        let mut result  = String::with_capacity(8192);
        self.format_tree_node(&mut result,&self.root_node, compact);
        result
    }

    pub fn format_tree_node(&self, result: &mut String, nodeid: &NodeId, compact: bool) {
        let node = self.get_node(&nodeid);
        if compact {
            result.push_str(&node.data.depth.to_string());
            result.push_str(",");
        } else {
            for x in 0..node.data.depth {
                result.push_str("  ");
            }
        }
        let mut call_duration = node.data.call_duration;
        //sum all children duration of root
        if nodeid.index == 0 {
            for child in node.children.values() {
                call_duration += self.get_node(&child).data.call_duration;
            }
        }else {

        }

        //"depth, call_name, calls, duration\n"
        //let duration = call_duration as f64/1000_000.0;
        let duration = call_duration/1000_000;
        result.push_str(&node.data.name);
        result.push_str(",");
        result.push_str(&node.data.call_count.to_string());
        result.push_str(",");
        result.push_str(&duration.to_string());
        result.push_str("\n");

        for child in node.children.values() {
            self.format_tree_node(result,&child, compact);
        }
    }

    //fast generate display tree
    pub fn to_tree(&self) -> tree::TreeNode {
        self.build_node(&self.root_node)
    }

    fn build_node(&self, nodeid: &NodeId) -> tree::TreeNode {
        let node = self.get_node(&nodeid);

        let mut children = vec![];
        for child in node.children.values() {
            children.push(Box::new(self.build_node(&child)));
        }
        //TODO
        let id = nodeid.index as i64;
        let label = node.data.name.to_string();
        let calls = node.data.call_count as i64;
        let cpu = node.data.call_cpu / 1000; //micros
        let duration = node.data.call_duration; //mills
        tree::TreeNode{ parent: None, id, label, calls, cpu, duration, start_time: 0, children, depth: 0 }
    }

    pub fn get_top_node(&self) -> &TreeNode {
        &self.nodes[self.top_call_stack_node.index]
    }

    pub fn get_mut_top_node(&mut self) -> &mut TreeNode {
        self.nodes.get_mut(self.top_call_stack_node.index).unwrap()
    }

    pub fn get_node(&self, node_id: &NodeId) -> &TreeNode {
        &self.nodes[node_id.index]
    }

    pub fn get_mut_node(&mut self, node_id: &NodeId) -> &mut TreeNode {
        &mut self.nodes[node_id.index]
    }

    pub fn get_root_node(&self) -> &TreeNode {
        &self.nodes[self.root_node.index]
    }
}

#[derive(Clone)]
pub struct NodeData {
    pub node_id: NodeId,
    pub depth: u32, // move to TreeNode
    pub name: String,
//    path: String,
    pub call_count: u32, // call count
    pub call_duration: i64, // call duration
    pub call_cpu: i64,
    pub children_size: u32 //children size
}

#[derive(Clone, Copy)]
pub struct NodeId {
    index: usize,
}

#[derive( Clone)]
pub struct TreeNode {
    id: u64,
    pub data: NodeData,
    parent: Option<NodeId>,
    children: HashMap<u64, NodeId>
}

impl TreeNode {

    pub fn newRootNode(name: &str) -> TreeNode {
        TreeNode{
            id: 0,
            data : NodeData {
                node_id: NodeId{index:0},
                depth: 0,
                name: name.to_string(),
//                path: name.to_string(),
                call_count: 0,
                call_duration: 0,
                call_cpu: 0,
                children_size: 0,
            },
            parent: None,
            children: HashMap::new()
        }
    }

    pub fn newCallNode(parentNode: &mut TreeNode, next_index: usize, method_id: &JavaMethod) -> TreeNode {

        //call path
//        let mut path = parentNode.data.path.to_string();
//        path += ";";
//        path += name.as_str();

        let node_id = NodeId{index:next_index};

        parentNode.children.insert(*method_id as u64, node_id.clone());
        parentNode.data.children_size += 1;

        TreeNode{
            id: *method_id as u64,
            data : NodeData {
                node_id: node_id,
                name: String::new(),
//                path: path.to_string(),
                depth: parentNode.data.depth + 1,
                call_count: 0,
                call_duration: 0,
                call_cpu: 0,
                children_size: 0,
            },
            parent: Some(parentNode.data.node_id.clone()),
            children: HashMap::new(),
        }

    }

    fn find_child(&self,  method_id: &JavaMethod) -> Option<&NodeId> {
        let key = *method_id as u64;
        self.children.get(&key)
    }

}