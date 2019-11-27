
use ::sample::*;
use std::collections::{HashMap, HashSet};

pub struct MethodCallFeature {
    name: String,
    desc: String,
    includes: Vec<String>,
    excludes: Vec<String>,
    methods: HashMap<i64, String>,
}

#[derive(Clone, Serialize)]
pub struct CallInfo {
    pub method_id: i64,
    pub full_name: String,
    pub features: Vec<String>
}

#[derive(Clone, Serialize)]
pub struct MethodCallGroup {
    group_id: i64,
    method_calls: Vec<Box<MethodCall>>,
    //分组的特征: Redis, MySQL,..
    features: Vec<String>,
    //分组公共的特征调用栈
    pub call_stack: Vec<CallInfo>,
    //公共调用栈的方法id 集合，用于快速比较
    #[serde(skip_serializing)]
    pub call_stack_ids: HashSet<i64>,
    #[serde(skip_serializing)]
    primary_stacks: Vec<i64>,
}

pub struct MethodAnalysis {
    pub call_groups: Vec<MethodCallGroup>,
    pub features: Vec<MethodCallFeature>,
    group_id_seq: i64
}

impl MethodAnalysis {

    pub fn new() -> MethodAnalysis {
        MethodAnalysis {
            call_groups: vec![],
            features: vec![],
            group_id_seq: 0
        }
    }

    pub fn add_method_call(&mut self, method_call: Box<MethodCall>) {
//        //找到最高相似度的分组
//        let (mut max_match_count, mut match_group) = (0, None);
//        for group in &mut self.call_groups {
//            let match_count = get_match_count(&method_call.primary_stacks, &group.call_stack_ids);
//            if match_count > max_match_count {
//                max_match_count = match_count;
//                match_group = Some(group);
//            }
//        }
//        //如果找到满足要求的分组，合并之
//        //TODO 要分别计算两个分组的相似度，避免某个方法是其中一个子集时也合并到一起的情况
//        let min_similarity = 0.9;
//        if max_match_count >= (method_call.primary_stacks.len() as f64 * min_similarity) as u64 {
//            if let Some(group) = match_group {
//                group.method_calls.push(method_call);
//                return;
//            }
//        }

        //合并相同特征栈的方法调用为一组
        for group in &mut self.call_groups {
            if group.primary_stacks.eq(&method_call.primary_stacks) {
                group.method_calls.push(method_call);
                return;
            }
        }

        //add new group
        let size = method_call.primary_stacks.len();
        let mut call_stack_ids = HashSet::with_capacity(size);
        let mut call_stack = Vec::with_capacity(size);
        for method_id in &method_call.primary_stacks {
            call_stack_ids.insert(*method_id);
            call_stack.push(CallInfo{
                method_id: *method_id,
                full_name: "".to_string(),
                features: vec![]
            })
        }
        let primary_stacks = method_call.primary_stacks.clone();

        self.group_id_seq += 1;
        let mut match_group = MethodCallGroup {
            group_id: self.group_id_seq,
            method_calls: vec![method_call],
            features: vec![],
            call_stack,
            call_stack_ids,
            primary_stacks,
        };
        self.call_groups.push(match_group);
    }

    pub fn get_method_groups(&self) -> &[MethodCallGroup] {
        &self.call_groups
    }
}

fn get_match_count(primary_stacks: &[i64], call_stack_ids: &HashSet<i64>) -> u64 {
    let mut count = 0;
    for method_id in primary_stacks {
        if call_stack_ids.contains(method_id) {
            count +=1;
        }
    }
    count
}