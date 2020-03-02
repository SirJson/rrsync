//! Synchronization between a ssh client and an ssh server.

use cdchunking::{Chunker, ZPAQ};
use dirs;
use ssh2::Session;
use std::cell::RefCell;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::VecDeque;
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::net::TcpStream;
use std::ops::Not;
use std::path::Path;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::index::{Index, IndexTransaction, MAX_BLOCK_SIZE, ZPAQ_BITS};
use crate::sync::{IndexEvent, Sink, SinkWrapper, Source, SourceWrapper};
use crate::{Error, HashDigest};

pub struct SshSourceWrapper {
    index: Index,
    path: PathBuf,
}

impl SshSourceWrapper {
    pub fn new(location: &SshLocation) -> Result<SshSourceWrapper, Error> {
        let tcp = TcpStream::connect(location.host).unwrap();
        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();
        let mut private_key = dirs::home_dir().unwrap();
        private_key.push(".ssh/id_rsa");
        let mut public_key = dirs::home_dir().unwrap();
        public_key.push(".ssh/id_rsa.pub");
        /*
        let mut index = Index::open(&path.join(".rrsync.idx"))?;
        {
            let mut tx = index.transaction()?;
            info!("Indexing source into {:?}...", path.join(".rrsync.idx"));
            tx.index_path(path)?;
            tx.remove_missing_files(path)?;
            tx.commit()?;
        }
        let path = path.to_path_buf();
        Ok(FsSourceWrapper { index, path })*/

        if let Err(e) = sess.userauth_pubkey_file(
            location.user.unwrap_or_default(""),
            Some(public_key.as_path()),
            private_key.as_path(),
            Some("clonknoob"),
        ) {
            println!("Oh no {:?}", e);
            std::process::exit(1);
        }
        println!("AUTH = {}", sess.authenticated());
    }
}

impl SourceWrapper for SshSourceWrapper {
    fn open<'a>(&'a mut self) -> Result<Box<dyn Source + 'a>, Error> {
        Ok(Box::new(FsSource::new(
            self.index.transaction()?,
            &self.path,
        )?))
    }
}
