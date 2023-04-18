use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

// We'll use this type alias to denote what type of data will be used to send to each Worker
// In this case, we have a function (closure) that will run once
type Job = Box<dyn FnOnce() + Send + 'static>;

// Our ThreadPool object contains a list of Workers, as well as a
// mpsc::Sender, which tells the threads what kind of data that they'll
// expect to be sent through the Sender's channel, to the receiving end
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

// Each Worker will have a unique id to identify each one (for debugging or logging)
// as well as a handle (thread) to run.
struct Worker {
    id: usize,
    handle: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new "Worker" which should be added to our ThreadPool
    ///
    /// id is a unique identifier (for debugging or logging purposes)
    ///
    /// receiver expects the receiver from the corresponding mpsc::Sender
    /// which contains the Job (function/closure) that the Worker should run
    /// when it receives a request through the receiver.
    ///
    /// If an Err returns from the receiver, that means the Worker/thread
    /// should be shut down
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job! Executing...");
                    job();
                }
                Err(_) => {
                    println!("Worker {id} shutting down");
                    break;
                }
            }
        });

        Worker {
            id,
            handle: Some(handle),
        }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool
    ///
    /// The numThreads is the number of available threads in the pool
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero or less
    pub fn new(num_threads: usize) -> ThreadPool {
        assert!(num_threads > 0);

        // Create a channel, which provides a Sender/Receiver, and allows us to send information
        // (in our case, a Job object/type) to our Workers through the receiver
        let (sender, receiver) = mpsc::channel();

        // We need to wrap our receiver in an Arc<Mutex<T>>
        //   Arc<T>   = Allows us to have multiple of the same reference, even though we can only have one receiver
        //   Mutex<T> = Only lets one of the receiver references be used at a time, and other references to the same receiver
        //              will have to wait until the previous one has finished (let go of the lock/mutex)
        let receiver = Arc::new(Mutex::new(receiver));

        // Create our list of Workers, giving each one a reference to the receiver using Arc::clone()
        // to create a new reference to the same object for each Worker
        // Even though all of the Workers have the same receiver, the Mutex the receiver is wrapped in
        // will allow only one of the Workers to access it at a time.
        let mut workers = Vec::with_capacity(num_threads);
        for id in 0..num_threads {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Takes a function/closure, and gives it to a thread in the ThreadPool to run
    ///
    /// f: A function/closure, which should only run once
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // The function/closure being sent to our execute function needs to be wrapped
        // in a Box, to match the Job type which the send function will be expecting, due to the
        // type definition of the "sender" -> mpsc::Sender<Job>
        let job = Box::new(f);

        // Send our job using the "sender" on our ThreadPool, which will send the Job to the
        // corresponding receiver(s). Each of the workers will receive a request, but the Mutex
        // on the receiver makes sure that only one Worker can accept and process the request.
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender before stopping each of the workers (who each have the corresponding receiver)
        // so that the jobs don't wait forever and never stop, and no more requests can come in
        drop(self.sender.take());

        // Then, we'll wait for each worker to finish their request, and then exit each of them
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(handle) = worker.handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
