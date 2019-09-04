

fn main() {

}


pub struct TestClient {
    woker : TaskWorker,

}

impl TestClient {
    fn new() -> TestClient {

    }
}


struct Job {
    id: i32,
    name: String
}
struct TaskWorker {
    queue: VecDeque<Job>,
}

impl TaskWorker {

    fn addJob(&mut self, job: Job) {
        self.queue.push_back(job);
    }

    fn consumerJob(&mut self) {
       let job = self.queue.pop_front();
    }

    fn event_loop(&mut self) {
        loop {
            self.consumerJob();

        }
    }
}