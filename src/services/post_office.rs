use alloc;
use core::{
    arch::asm,
    cell::{Cell, RefCell},
};
use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::exception;

use crate::constants::MAX_TASKS;
use crate::debug;
use crate::services::task::Task;
use crate::sync::{NakedMutex, Spinlock};
use alloc::collections::{BTreeMap, VecDeque};
use alloc::string::String;
use alloc::vec::Vec;

pub(crate) static POST_OFFICE: Spinlock<RefCell<Option<PostOffice>>> =
    Spinlock::new(RefCell::new(None));

#[derive(Debug)]
pub enum PostOfficeError {
    AlreadyInitialized,
    NotInitialized,
    MailboxNotFound,
    MailboxTaskNameAlreadyUsed,
}

pub struct PostOffice {
    mailboxes: BTreeMap<usize, Mailboxes>,
    name_to_idx: BTreeMap<String, usize>,
}

impl PostOffice {
    fn new() -> Self {
        PostOffice {
            mailboxes: BTreeMap::new(),
            name_to_idx: BTreeMap::new(),
        }
    }

    pub fn init() -> Result<(), PostOfficeError> {
        if POST_OFFICE.lock().borrow().is_some() {
            return Err(PostOfficeError::AlreadyInitialized);
        }

        // I don't think this needs to be a critical section at least for the time being
        let post_office = PostOffice::new();
        POST_OFFICE.lock().borrow_mut().replace(post_office);

        debug!("Post Office initialization complete!");
        Ok(())
    }

    pub(crate) fn register_mailbox(
        task_index: usize,
        task_name: &str,
    ) -> Result<(), PostOfficeError> {
        if let Some(post_office) = POST_OFFICE.lock().borrow_mut().as_mut() {
            let task_string = String::from(task_name);

            post_office.mailboxes.insert(task_index, Mailboxes::new());
            if post_office.name_to_idx.get(&task_string).is_none() {
                post_office
                    .name_to_idx
                    .insert(String::from(task_name), task_index);

                debug!("New mailbox registered : {} | {}", task_index, task_name);
                Ok(())
            } else {
                Err(PostOfficeError::MailboxTaskNameAlreadyUsed)
            }
        } else {
            Err(PostOfficeError::NotInitialized)
        }
    }

    pub fn send(&self, msg: MailboxMsg) -> Result<(), PostOfficeError> {
        if let Some(mailboxes) = self.mailboxes.get(&msg.to_task) {
            mailboxes.incoming.borrow_mut().push_front(msg);

            Ok(())
        } else {
            Err(PostOfficeError::MailboxNotFound)
        }
    }

    pub fn send_to_task_by_name(task_name: &str, data: Vec<u8>) -> Result<(), PostOfficeError> {
        if let Some(post_office) = POST_OFFICE.lock().borrow().as_ref() {
            let msg = MailboxMsg {
                to_task: *post_office.name_to_idx.get(task_name).unwrap(),
                from_task: 0,
                data: data,
            };

            if let Some(mailboxes) = post_office.mailboxes.get(&msg.to_task) {
                mailboxes.incoming.borrow_mut().push_front(msg);

                Ok(())
            } else {
                Err(PostOfficeError::MailboxNotFound)
            }
        } else {
            Err(PostOfficeError::NotInitialized)
        }
    }

    pub fn recv(task_idx: usize) -> Result<Option<MailboxMsg>, PostOfficeError> {
        if let Some(post_office) = POST_OFFICE.lock().borrow().as_ref() {
            if let Some(mailboxes) = post_office.mailboxes.get(&task_idx) {
                Ok(mailboxes.incoming.borrow_mut().pop_front())
            } else {
                Err(PostOfficeError::MailboxNotFound)
            }
        } else {
            Err(PostOfficeError::NotInitialized)
        }
    }

    pub fn recv_by_name(task_name: String) -> Result<Option<MailboxMsg>, PostOfficeError> {
        if let Some(post_office) = POST_OFFICE.lock().borrow().as_ref() {
            if let Some(mailboxes) = post_office
                .mailboxes
                .get(&post_office.name_to_idx.get(&task_name).unwrap())
            {
                Ok(mailboxes.incoming.borrow_mut().pop_front())
            } else {
                Err(PostOfficeError::MailboxNotFound)
            }
        } else {
            Err(PostOfficeError::NotInitialized)
        }
    }
}

pub struct MailboxMsg {
    to_task: usize,
    from_task: usize,
    data: Vec<u8>,
}

impl MailboxMsg {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

pub struct Mailboxes {
    incoming: RefCell<VecDeque<MailboxMsg>>,
    // from_task: RefCell<VecDeque<MailboxMsg>>,
}

impl Mailboxes {
    fn new() -> Self {
        Self {
            incoming: RefCell::new(VecDeque::new()),
            // from_task: RefCell::new(VecDeque::new()),
        }
    }
}
