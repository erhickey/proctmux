use std::sync::{Arc, Mutex, mpsc::Receiver};
use std::thread::spawn;

// use sysinfo::{System, SystemExt, Pid};

use crate::controller::Controller;

// pub fn start_watching_pids(controller: Arc<Mutex<Controller>>) {
//     spawn(move || {
//         let mut sys = System::new_all();
//         loop {
//             info!("Checking for dead processes");
//             sys.refresh_processes();
//             let system_pids = sys.processes();
//             let mut locked_controller= controller.lock().unwrap();
//             let proc_to_pid =  locked_controller.get_processes_to_pid();
//             proc_to_pid.iter().for_each(|(process_index, pid)| {
//                 if let Some(pid) = pid {
//                     if !system_pids.contains_key(&Pid::from(*pid as usize)) {
//                         info!("Process {} is dead - marking as terminated", process_index);
//                         let _ = locked_controller.on_process_terminated(*process_index);
//                     }
//                 }
//             });
//             drop(locked_controller);
//             info!("done checking for dead processes - sleeping");
//             std::thread::sleep(std::time::Duration::from_millis(5000));
//         }
//     });
// }

pub fn receive_dead_pids(receiver: Receiver<i32>, controller: Arc<Mutex<Controller>>) {
    spawn(move || {
        for pid in receiver {
            controller.lock().unwrap().on_pid_terminated(pid).unwrap();
        }
    });
}
