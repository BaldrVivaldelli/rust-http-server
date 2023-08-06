use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;

pub struct ThreadPool{
    workers: Vec<Worker>,
    sender: mpsc::Sender<Messaje>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Messaje{
    NewJob(Job),
    Terminate
}

impl ThreadPool{
    ///Create a new ThreadPool
    ///Panic if 0 threads are avaliable
    ///
    pub fn new(size: usize)-> ThreadPool{

        assert!(size > 0);

        let (sender, reciver) = mpsc::channel();

        let reciver = Arc::new(Mutex::new(reciver));
        let mut workers =Vec::with_capacity(size);
        for id in 0..size{
            workers.push(Worker::new(id,Arc::clone(&reciver)))
        }
        ThreadPool {workers,sender}
    }

    pub fn execute<F>(&self, f:F)
    where
        F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(Messaje::NewJob(job)).unwrap();
    }

}
impl Drop for ThreadPool{
    fn drop(&mut self){

        for _ in &self.workers{
            self.sender.send(Messaje::Terminate).unwrap()
        }

        for worker in & mut self.workers{
            println!("Shutting down {}",worker.id);
            if let Some(thread) = worker.thread.take(){
                thread.join().unwrap();
            }

        }
    }
}


//monad JoinHandle<()>
struct Worker{
    id:usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {
    fn new(id: usize, reciver: Arc<Mutex<Receiver<Messaje>>>) -> Worker{
        let thread = thread::spawn(move || loop{
            //todo: cambiar esto
            let message = reciver.lock().unwrap().recv().unwrap();
            match message {
                Messaje::NewJob( job) =>{
                    println!("Worker {} got the job; executing job {}",id,id);
                    job()
                }
                Messaje::Terminate =>{
                println!("The worker {} was call to be terminated",id);
                }

            }
        });
        Worker{id, thread: Some(thread)}
    }
}