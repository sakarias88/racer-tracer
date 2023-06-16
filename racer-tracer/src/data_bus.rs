use std::{
    sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender},
    time::Duration,
};

use bus::{Bus, BusReader};

use crate::error::TracerError;

#[derive(Clone)]
pub struct DataWriter<T> {
    channel_name: String,
    sender: Sender<T>,
}

impl<T> DataWriter<T> {
    fn new(channel_name: String, sender: Sender<T>) -> Self {
        Self {
            channel_name,
            sender,
        }
    }

    pub fn write(&self, action: T) -> Result<(), TracerError> {
        self.sender
            .send(action)
            .map_err(|e| TracerError::BusWriteError(self.channel_name.clone(), e.to_string()))
    }
}

pub struct DataReader<T: Clone + Sync> {
    channel_name: String,
    reader: BusReader<T>,
}

impl<T: Clone + Sync> DataReader<T> {
    fn new(channel_name: String, reader: BusReader<T>) -> Self {
        Self {
            channel_name,
            reader,
        }
    }

    pub fn get_messages(&mut self) -> Result<Vec<T>, TracerError> {
        // Note: This does not block or suspend which can be nice?
        // Can also not be nice since it's never suspending the thread
        // implicitly creating a busy update loop where nothing
        // actually happens.
        // Probably better to make it blocking, ie. suspend the thread
        // until something actually happens. Will look into this later.

        // TODO: This code could possible be expressed in a nicer manner.
        let mut buf: Vec<T> = vec![];
        let mut res = self.reader.recv_timeout(Duration::from_millis(0));
        while let Ok(action) = res {
            buf.push(action);
            res = self.reader.recv_timeout(Duration::from_millis(0));
        }

        match res {
            Ok(_) => Ok(buf),
            Err(e) if e == RecvTimeoutError::Timeout => Ok(buf),
            Err(e) => Err(TracerError::BusReadError(
                self.channel_name.clone(),
                e.to_string(),
            )),
        }
    }
}

pub struct DataBus<T: Clone + Sync> {
    channel_name: String,
    receiver: Receiver<T>,
    bus: Bus<T>,
    data_writer: Sender<T>,
}

impl<T: Clone + Sync> DataBus<T> {
    pub fn new<U: AsRef<str>>(channel_name: U) -> Self {
        let (sender, receiver) = channel();
        Self {
            channel_name: channel_name.as_ref().to_string(),
            receiver,
            data_writer: sender,
            bus: Bus::new(1024),
        }
    }

    pub fn update(&mut self) -> Result<(), TracerError> {
        let mut res: Result<T, TracerError> = self
            .receiver
            .recv_timeout(Duration::from_millis(0))
            .map_err(|e| e.into());

        while let Ok(action) = res {
            res = self
                .bus
                .try_broadcast(action)
                .map_err(|_| TracerError::BusUpdateError(self.channel_name.clone()))
                .and_then(|_| {
                    self.receiver
                        .recv_timeout(Duration::from_millis(0))
                        .map_err(|e| e.into())
                });
        }

        match res {
            Ok(_) => Ok(()),
            Err(e) if e == TracerError::BusTimeoutError() => Ok(()),
            Err(e) => Err(TracerError::RecieveError(e.to_string())),
        }
    }

    pub fn get_reader(&mut self) -> DataReader<T> {
        DataReader::new(self.channel_name.clone(), self.bus.add_rx())
    }

    pub fn get_writer(&self) -> DataWriter<T> {
        DataWriter::new(self.channel_name.clone(), self.data_writer.clone())
    }
}

impl From<RecvTimeoutError> for TracerError {
    fn from(e: RecvTimeoutError) -> Self {
        match e {
            RecvTimeoutError::Timeout => TracerError::BusTimeoutError(),
            RecvTimeoutError::Disconnected => TracerError::BusUpdateError(e.to_string()),
        }
    }
}
