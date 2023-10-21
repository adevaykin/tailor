pub mod filewatchclient;
pub mod dirwatchclient;

pub trait WatchClient {
    fn start(&mut self, client_id: i32);
}