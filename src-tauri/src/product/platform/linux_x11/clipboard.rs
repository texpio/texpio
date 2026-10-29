//! Bounded ICCCM selection source. INCR, MULTIPLE and lazy/unmaterialized data refuse
//! restoration before erasure. A restored source remains alive until runtime stop.
use super::{fail, policy};
use crate::product::effects::ProductError;
use std::{
    collections::BTreeMap,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{
        Event,
        xfixes::{self, ConnectionExt as _},
        xproto::*,
    },
    rust_connection::RustConnection,
    wrapper::ConnectionExt as _,
};

#[derive(Clone)]
pub(super) struct Format {
    kind: Atom,
    format: u8,
    bytes: Vec<u8>,
}
pub(super) struct Clipboard {
    pub connection: Arc<RustConnection>,
    window: Window,
    selection: Atom,
    targets: Atom,
    property: Atom,
    incr: Atom,
    utf8: Atom,
    timestamp: Atom,
    multiple: Atom,
    source: BTreeMap<Atom, Format>,
    owner: Window,
    epoch: u32,
    acquired: Option<u32>,
}
impl Clipboard {
    pub fn new(connection: Arc<RustConnection>, screen: usize) -> Result<Self, ProductError> {
        let window = connection.generate_id().map_err(fail)?;
        let root = connection.setup().roots[screen].root;
        connection
            .create_window(
                0,
                window,
                root,
                0,
                0,
                1,
                1,
                0,
                WindowClass::INPUT_ONLY,
                0,
                &CreateWindowAux::new().event_mask(EventMask::PROPERTY_CHANGE),
            )
            .map_err(fail)?
            .check()
            .map_err(fail)?;
        let atom = |name: &[u8]| -> Result<Atom, ProductError> {
            Ok(connection
                .intern_atom(false, name)
                .map_err(fail)?
                .reply()
                .map_err(fail)?
                .atom)
        };
        let selection = atom(b"CLIPBOARD")?;
        connection
            .xfixes_select_selection_input(
                window,
                selection,
                xfixes::SelectionEventMask::SET_SELECTION_OWNER
                    | xfixes::SelectionEventMask::SELECTION_WINDOW_DESTROY
                    | xfixes::SelectionEventMask::SELECTION_CLIENT_CLOSE,
            )
            .map_err(fail)?
            .check()
            .map_err(fail)?;
        let result = Self {
            window,
            selection,
            targets: atom(b"TARGETS")?,
            property: atom(b"TEXPIO_TRANSFER")?,
            incr: atom(b"INCR")?,
            utf8: atom(b"UTF8_STRING")?,
            timestamp: atom(b"TIMESTAMP")?,
            multiple: atom(b"MULTIPLE")?,
            source: BTreeMap::new(),
            owner: 0,
            epoch: 0,
            acquired: None,
            connection,
        };
        Ok(result)
    }
    pub fn pump(&mut self) -> Result<(), ProductError> {
        // Bound work per iteration; malicious request floods cannot starve cancellation.
        for _ in 0..64 {
            let Some(event) = self.connection.poll_for_event().map_err(fail)? else {
                return self.connection.flush().map_err(fail);
            };
            self.event(event)?;
        }
        // A flood must not hide a newer owner/timestamp behind a truncated drain.
        Err(ProductError::Clipboard)
    }
    fn event(&mut self, event: Event) -> Result<(), ProductError> {
        match event {
            Event::XfixesSelectionNotify(e) if e.selection == self.selection => {
                self.owner = e.owner;
                self.epoch = e.selection_timestamp;
                if e.owner != self.window {
                    self.acquired = None;
                    self.source.clear();
                }
            }
            Event::SelectionClear(e) if e.selection == self.selection => {
                self.acquired = None;
                self.source.clear();
            }
            Event::SelectionRequest(e) if e.selection == self.selection => {
                let property = if e.property == 0 {
                    e.target
                } else {
                    e.property
                };
                let mut accepted = false;
                if e.owner == self.window && e.target == self.targets {
                    let mut atoms: Vec<Atom> = self.source.keys().copied().collect();
                    atoms.extend([self.targets, self.timestamp]);
                    accepted = self
                        .connection
                        .change_property32(
                            PropMode::REPLACE,
                            e.requestor,
                            property,
                            AtomEnum::ATOM,
                            &atoms,
                        )
                        .map_err(fail)?
                        .check()
                        .is_ok();
                } else if e.owner == self.window && e.target == self.timestamp {
                    accepted = self
                        .connection
                        .change_property32(
                            PropMode::REPLACE,
                            e.requestor,
                            property,
                            AtomEnum::INTEGER,
                            &[self.epoch],
                        )
                        .map_err(fail)?
                        .check()
                        .is_ok();
                } else if let Some(format) = self.source.get(&e.target) {
                    // Request length is bounded on capture. Do not implement unbounded INCR sends.
                    accepted = self
                        .connection
                        .change_property(
                            PropMode::REPLACE,
                            e.requestor,
                            property,
                            format.kind,
                            format.format,
                            (format.bytes.len() / (usize::from(format.format) / 8)) as u32,
                            &format.bytes,
                        )
                        .map_err(fail)?
                        .check()
                        .is_ok();
                }
                let notify = SelectionNotifyEvent {
                    response_type: SELECTION_NOTIFY_EVENT,
                    sequence: 0,
                    time: e.time,
                    requestor: e.requestor,
                    selection: e.selection,
                    target: e.target,
                    property: if accepted { property } else { 0 },
                };
                self.connection
                    .send_event(false, e.requestor, EventMask::NO_EVENT, notify)
                    .map_err(fail)?
                    .ignore_error();
            }
            _ => {}
        }
        Ok(())
    }
    fn receive(&mut self, target: Atom, deadline: Instant) -> Result<Format, ProductError> {
        self.connection
            .delete_property(self.window, self.property)
            .map_err(fail)?
            .check()
            .map_err(fail)?;
        self.connection
            .convert_selection(
                self.window,
                self.selection,
                target,
                self.property,
                x11rb::CURRENT_TIME,
            )
            .map_err(fail)?;
        self.connection.flush().map_err(fail)?;
        while Instant::now() < deadline {
            if let Some(event) = self.connection.poll_for_event().map_err(fail)? {
                if let Event::SelectionNotify(e) = &event
                    && e.requestor == self.window
                    && e.selection == self.selection
                    && e.target == target
                {
                    if e.property == 0 {
                        return Err(ProductError::Clipboard);
                    }
                    let data = self
                        .connection
                        .get_property(
                            true,
                            self.window,
                            self.property,
                            AtomEnum::ANY,
                            0,
                            (policy::MAX_BYTES / 4 + 1) as u32,
                        )
                        .map_err(fail)?
                        .reply()
                        .map_err(fail)?;
                    if data.type_ == self.incr
                        || data.type_ == 0
                        || data.bytes_after != 0
                        || data.value.len() > policy::MAX_BYTES
                        || ![8, 16, 32].contains(&data.format)
                    {
                        return Err(ProductError::Clipboard);
                    }
                    return Ok(Format {
                        kind: data.type_,
                        format: data.format,
                        bytes: data.value,
                    });
                }
                self.event(event)?;
            } else {
                thread::sleep(Duration::from_millis(2));
            }
        }
        Err(ProductError::Clipboard)
    }
    fn with_server_grab<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, ProductError>,
    ) -> Result<T, ProductError> {
        self.connection
            .grab_server()
            .map_err(fail)?
            .check()
            .map_err(fail)?;
        let result = operation(self);
        let release = self
            .connection
            .ungrab_server()
            .map_err(fail)?
            .check()
            .map_err(fail);
        release?;
        result
    }
    // Caller holds the server grab: owner and all queued XFixes changes describe
    // one instant. The initial epoch may be zero, but later changes are tracked.
    fn observed_stamp(&mut self) -> Result<policy::SelectionStamp, ProductError> {
        let owner = self
            .connection
            .get_selection_owner(self.selection)
            .map_err(fail)?
            .reply()
            .map_err(fail)?
            .owner;
        self.pump()?;
        Ok(policy::SelectionStamp {
            owner,
            epoch: self.epoch,
        })
    }
    pub fn acquisition_token(&mut self) -> Result<policy::SelectionStamp, ProductError> {
        self.with_server_grab(Self::observed_stamp)
    }
    pub fn snapshot(
        &mut self,
    ) -> Result<(BTreeMap<Atom, Format>, policy::SelectionStamp), ProductError> {
        let token = self.acquisition_token()?;
        let owner = token.owner;
        if owner == self.window {
            return Ok((self.source.clone(), token));
        }
        if owner == 0 {
            return Ok((BTreeMap::new(), token));
        }
        let deadline = Instant::now() + Duration::from_millis(750);
        let formats = self.receive(self.targets, deadline)?;
        if formats.format != 32
            || formats.kind != u32::from(AtomEnum::ATOM)
            || formats.bytes.len() > 4 * (policy::MAX_FORMATS + 4)
        {
            return Err(ProductError::Clipboard);
        }
        let mut snapshot = BTreeMap::new();
        let mut bytes = 0;
        for chunk in formats.bytes.as_chunks::<4>().0 {
            let target = u32::from_ne_bytes(*chunk);
            if [self.targets, self.timestamp, self.multiple].contains(&target) {
                continue;
            }
            if target == self.incr {
                return Err(ProductError::Clipboard);
            }
            let value = self.receive(target, deadline)?;
            if value.bytes.len() > self.connection.maximum_request_bytes().saturating_sub(128) {
                return Err(ProductError::Clipboard);
            }
            bytes += value.bytes.len();
            snapshot.insert(target, value);
            if !policy::snapshot_complete(snapshot.len(), bytes, true) {
                return Err(ProductError::Clipboard);
            }
        }
        let current = self.acquisition_token()?;
        if !policy::acquisition_current(token, current) {
            return Err(ProductError::Clipboard);
        }
        Ok((snapshot, token))
    }
    pub fn publish(
        &mut self,
        text: &str,
        token: policy::SelectionStamp,
    ) -> Result<(), ProductError> {
        if text.len() > self.connection.maximum_request_bytes().saturating_sub(128)
            || text.len() > policy::MAX_BYTES
        {
            return Err(ProductError::Clipboard);
        }
        let source = BTreeMap::from([(
            self.utf8,
            Format {
                kind: self.utf8,
                format: 8,
                bytes: text.as_bytes().to_vec(),
            },
        )]);
        self.with_server_grab(move |this| {
            if !policy::acquisition_current(token, this.observed_stamp()?) {
                return Err(ProductError::Clipboard);
            }
            // Comparison and ownership acquisition occur in the same server grab.
            // No copy between snapshot/preflight and publication may be overwritten.
            let previous = std::mem::replace(&mut this.source, source);
            let acquired = this
                .connection
                .set_selection_owner(this.window, this.selection, x11rb::CURRENT_TIME)
                .map_err(fail)
                .and_then(|cookie| cookie.check().map_err(fail));
            if let Err(error) = acquired {
                this.source = previous;
                return Err(error);
            }
            this.connection
                .get_input_focus()
                .map_err(fail)?
                .reply()
                .map_err(fail)?;
            this.pump()?;
            if this.owner != this.window {
                return Err(ProductError::Clipboard);
            }
            this.acquired = Some(this.epoch);
            Ok(())
        })
    }
    pub fn current(&mut self) -> Result<bool, ProductError> {
        self.connection
            .get_input_focus()
            .map_err(fail)?
            .reply()
            .map_err(fail)?;
        self.pump()?;
        let owner = self
            .connection
            .get_selection_owner(self.selection)
            .map_err(fail)?
            .reply()
            .map_err(fail)?
            .owner;
        Ok(self
            .acquired
            .is_some_and(|epoch| policy::lease_current(self.window, owner, epoch, self.epoch)))
    }
    pub fn restore(&mut self, snapshot: BTreeMap<Atom, Format>) -> Result<bool, ProductError> {
        // Server grab makes comparison + source publication atomic against another SetSelectionOwner.
        // Never reassign the previous owner's window: the old provider may already have exited.
        self.connection
            .grab_server()
            .map_err(fail)?
            .check()
            .map_err(fail)?;
        let result = self.current().inspect(|&current| {
            if current {
                self.source = snapshot;
            }
        });
        let release = self
            .connection
            .ungrab_server()
            .map_err(fail)?
            .check()
            .map_err(fail);
        release?;
        result
    }
}
impl Drop for Clipboard {
    fn drop(&mut self) {
        let _ = self.connection.destroy_window(self.window);
        let _ = self.connection.flush();
    }
}
