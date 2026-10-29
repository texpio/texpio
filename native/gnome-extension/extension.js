// Original implementation against GNOME Shell/Mutter 46 APIs; no upstream runtime copied.
import Gio from 'gi://Gio'
import GLib from 'gi://GLib'
import Meta from 'gi://Meta'
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js'
import * as Main from 'resource:///org/gnome/shell/ui/main.js'
import * as Keyboard from 'resource:///org/gnome/shell/ui/status/keyboard.js'
import {
  ATTESTATION_DIRECTORY,
  MAX_ATTESTATION,
  MAX_PROC_STAT,
  trustedDirectory,
  trustedFile,
  validateRecord,
} from './auth-policy.js'

const PATH = '/org/texpio/GnomeBridge'
const NAME = 'org.texpio.GnomeBridge'
const MAX = 64 * 1024
const TEXT_MAX = 16 * 1024
const XML = `<node><interface name="org.texpio.GnomeBridge1">
<method name="Exchange"><arg type="s" direction="in"/><arg type="s" direction="out"/></method>
</interface></node>`
const METADATA = 'unix::uid,unix::mode,unix::nlink,standard::is-symlink,standard::size'
function metadata(info) {
  for (const name of ['unix::uid', 'unix::mode'])
    if (!info.has_attribute(name)) throw new Error('metadata')
  return {
    uid: info.get_attribute_uint32('unix::uid'),
    mode: info.get_attribute_uint32('unix::mode'),
    symlink: info.get_is_symlink(),
    links: info.has_attribute('unix::nlink') ? info.get_attribute_uint32('unix::nlink') : 0,
  }
}
function trustedParent(path) {
  const info = Gio.File.new_for_path(path).query_info(
    METADATA,
    Gio.FileQueryInfoFlags.NOFOLLOW_SYMLINKS,
    null,
  )
  if (!trustedDirectory(metadata(info))) throw new Error('directory')
}
function readBounded(path, limit, attested = false) {
  const file = Gio.File.new_for_path(path)
  if (attested) {
    const info = file.query_info(METADATA, Gio.FileQueryInfoFlags.NOFOLLOW_SYMLINKS, null)
    if (!trustedFile(metadata(info)) || info.get_size() <= 0 || info.get_size() > limit)
      throw new Error('file')
  }
  const stream = file.read(null)
  try {
    // The immutable root parent prevents a session user from swapping the path
    // between the no-follow path check and open. Recheck the opened file too.
    if (attested) {
      const info = stream.query_info(METADATA, null)
      if (!trustedFile(metadata(info)) || info.get_size() <= 0 || info.get_size() > limit)
        throw new Error('file')
    }
    const buffer = new Uint8Array(limit + 1)
    let used = 0
    while (used <= limit) {
      const bytes = stream.read_bytes(limit + 1 - used, null).get_data()
      if (bytes.length === 0) break
      buffer.set(bytes, used)
      used += bytes.length
      if (used > limit) throw new Error('file limit')
    }
    return new TextDecoder('utf-8', { fatal: true }).decode(buffer.subarray(0, used))
  } finally {
    stream.close(null)
  }
}
function call(method, sender) {
  return new Promise((resolve, reject) =>
    Gio.DBus.session.call(
      'org.freedesktop.DBus',
      '/org/freedesktop/DBus',
      'org.freedesktop.DBus',
      method,
      new GLib.Variant('(s)', [sender]),
      new GLib.VariantType('(u)'),
      Gio.DBusCallFlags.NONE,
      200,
      null,
      (connection, result) => {
        try {
          resolve(connection.call_finish(result).deep_unpack()[0])
        } catch (error) {
          reject(error)
        }
      },
    ),
  )
}
export default class TexpioBridge extends Extension {
  enable() {
    this._epoch = 1
    this._clipboardEpoch = 1
    this._leaseCounter = 0
    this._lease = null
    this._busy = false
    this._disabled = false
    this._signals = []
    this._cancellables = new Set()
    this._source = Keyboard.getInputSourceManager()
    this._settings = new Gio.Settings({ schema_id: 'org.gnome.desktop.input-sources' })
    this._selection = global.display.get_selection()
    const track = (object, signal, callback = () => this._epoch++) => {
      this._signals.push([object, object.connect(signal, callback)])
    }
    track(global.display, 'notify::focus-window')
    track(this._source, 'current-source-changed')
    track(this._settings, 'changed')
    track(Main.overview, 'showing')
    track(Main.overview, 'hiding')
    track(Main.sessionMode, 'updated')
    track(Main.screenShield, 'locked-changed', () => {
      this._epoch++
      if (Main.screenShield.locked) this._abandon()
    })
    track(this._selection, 'owner-changed', (_selection, type) => {
      if (type === Meta.SelectionType.CLIPBOARD) this._clipboardEpoch++
    })
    this._object = Gio.DBusExportedObject.wrapJSObject(XML, this)
    this._object.export(Gio.DBus.session, PATH)
    this._owner = Gio.bus_own_name_on_connection(
      Gio.DBus.session,
      NAME,
      Gio.BusNameOwnerFlags.NONE,
      null,
      () => this._abandon(),
    )
  }
  async _authorize(invocation) {
    const sender = invocation.get_sender()
    const [uid, pid] = await Promise.all([
      call('GetConnectionUnixUser', sender),
      call('GetConnectionUnixProcessID', sender),
    ])
    if (
      uid !== new Gio.Credentials().get_unix_user() ||
      !Number.isSafeInteger(pid) ||
      pid <= 0 ||
      pid > 0x7fffffff
    )
      throw new Error('caller')
    trustedParent('/run')
    trustedParent(ATTESTATION_DIRECTORY)
    const record = readBounded(`${ATTESTATION_DIRECTORY}/${pid}.json`, MAX_ATTESTATION, true)
    const stat = readBounded(`/proc/${pid}/stat`, MAX_PROC_STAT)
    if (!validateRecord(record, pid, uid, stat)) throw new Error('caller')
    return sender
  }
  _unlocked() {
    return (
      !this._disabled &&
      !Main.screenShield.locked &&
      !Main.sessionMode.isLocked &&
      Main.sessionMode.currentMode === 'user'
    )
  }
  _context() {
    const window = global.display.focus_window
    const source = this._source.currentSource
    return {
      version: 1,
      epoch: this._epoch,
      pid: window?.get_pid() ?? 0,
      unlocked: this._unlocked(),
      overview: Main.overview.visible,
      sourceType: source?.type ?? '',
      layout: source?.xkbId ?? '',
      options: this._settings.get_strv('xkb-options').join(','),
    }
  }
  async ExchangeAsync([raw], invocation) {
    if (this._busy || typeof raw !== 'string' || new TextEncoder().encode(raw).length > MAX) {
      invocation.return_dbus_error('org.texpio.Error.Refused', 'busy or frame limit')
      return
    }
    this._busy = true
    try {
      let sender
      try {
        sender = await this._authorize(invocation)
      } catch (_) {
        invocation.return_dbus_error(
          'org.texpio.Error.AuthenticationUnavailable',
          'native broker attestation required',
        )
        return
      }
      const request = JSON.parse(raw)
      if (request.version !== 1 || !this._unlocked()) throw new Error('session')
      let result
      switch (request.op) {
        case 'context':
          result = this._context()
          break
        case 'clipboardAcquire':
          result = await this._acquire(request, sender)
          break
        case 'clipboardCheck':
          result = { owned: this._owns(request.lease, sender) }
          break
        case 'clipboardRelease':
          result = this._release(request.lease, sender)
          break
        default:
          throw new Error('operation')
      }
      if (this._disabled) throw new Error('disabled')
      invocation.return_value(new GLib.Variant('(s)', [JSON.stringify(result)]))
    } catch (_) {
      // Never expose exception text: payloads and file names must stay private.
      invocation.return_dbus_error('org.texpio.Error.Refused', 'required evidence unavailable')
    } finally {
      this._busy = false
    }
  }
  _read(mime) {
    return new Promise((resolve, reject) => {
      const stream = Gio.MemoryOutputStream.new_resizable()
      const cancel = new Gio.Cancellable()
      this._cancellables.add(cancel)
      const timeout = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 200, () => {
        cancel.cancel()
        return GLib.SOURCE_REMOVE
      })
      this._selection.transfer_async(
        Meta.SelectionType.CLIPBOARD,
        mime,
        MAX + 1,
        stream,
        cancel,
        (selection, result) => {
          GLib.Source.remove(timeout)
          this._cancellables.delete(cancel)
          try {
            if (!selection.transfer_finish(result)) throw new Error('transfer')
            stream.close(null)
            const bytes = stream.steal_as_bytes()
            if (bytes.get_size() > MAX) throw new Error('limit')
            resolve(bytes)
          } catch (error) {
            reject(error)
          }
        },
      )
    })
  }
  async _acquire(request, sender) {
    if (this._lease || typeof request.text !== 'string' || typeof request.restore !== 'boolean')
      throw new Error('lease')
    const bytes = new TextEncoder().encode(request.text)
    if (bytes.length > TEXT_MAX) throw new Error('limit')
    const epoch = this._clipboardEpoch
    const formats = this._selection.get_mimetypes(Meta.SelectionType.CLIPBOARD) ?? []
    let snapshot = null
    // Meta.SelectionSourceMemory supplies one MIME. A multi-format owner must
    // be refused before erasure when restoration is requested. Never flatten
    // HTML/images/files into a text-only snapshot and call it complete.
    if (request.restore && formats.length > 0) {
      if (formats.length !== 1 || !['text/plain;charset=utf-8', 'text/plain'].includes(formats[0]))
        throw new Error('incomplete snapshot')
      snapshot = { mime: formats[0], bytes: await this._read(formats[0]) }
    }
    if (epoch !== this._clipboardEpoch || !this._unlocked()) throw new Error('owner changed')
    const source = Meta.SelectionSourceMemory.new('text/plain;charset=utf-8', new GLib.Bytes(bytes))
    this._selection.set_owner(Meta.SelectionType.CLIPBOARD, source)
    const lease = ++this._leaseCounter
    const watchdog = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1500, () => {
      this._release(lease, sender)
      return GLib.SOURCE_REMOVE
    })
    this._lease = {
      id: lease,
      sender,
      source,
      epoch: this._clipboardEpoch,
      snapshot,
      restore: request.restore,
      watchdog,
    }
    return { lease }
  }
  _owns(id, sender) {
    return (
      this._unlocked() &&
      this._lease?.id === id &&
      this._lease.sender === sender &&
      this._lease.epoch === this._clipboardEpoch
    )
  }
  _release(id, sender) {
    if (!this._lease || this._lease.id !== id || this._lease.sender !== sender)
      return { restored: false }
    const lease = this._lease
    const owned = this._owns(id, sender)
    this._lease = null
    if (lease.watchdog) GLib.Source.remove(lease.watchdog)
    if (owned && lease.restore) {
      if (lease.snapshot) {
        const previous = Meta.SelectionSourceMemory.new(lease.snapshot.mime, lease.snapshot.bytes)
        this._selection.set_owner(Meta.SelectionType.CLIPBOARD, previous)
      } else this._selection.unset_owner(Meta.SelectionType.CLIPBOARD, lease.source)
    }
    return { restored: owned && lease.restore }
  }
  _abandon() {
    for (const cancel of this._cancellables) cancel.cancel()
    if (this._lease) {
      const lease = this._lease
      this._lease = null
      if (lease.watchdog) GLib.Source.remove(lease.watchdog)
      // Conditional unset removes only this exact source, never a concurrent copy.
      this._selection.unset_owner(Meta.SelectionType.CLIPBOARD, lease.source)
    }
  }
  disable() {
    this._disabled = true
    this._abandon()
    for (const [object, id] of this._signals) object.disconnect(id)
    this._signals = []
    this._object?.unexport()
    this._object = null
    if (this._owner) Gio.bus_unown_name(this._owner)
    this._owner = 0
  }
}
