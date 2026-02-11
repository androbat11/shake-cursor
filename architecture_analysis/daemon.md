# Daemon

## Definition

A **daemon** is a background process that runs continuously without direct user interaction. It starts automatically (on boot or login), runs silently in the background, and provides a service for as long as the system is running.

The name "daemon" comes from Greek mythology — a daemon was a helpful spirit that worked in the background. In Linux, daemons are the invisible workers that keep your system functioning.

```
┌─────────────────────────────────────────────────────────────┐
│                    Real World Analogy                       │
│                                                             │
│   REGULAR PROGRAM:                                          │
│                                                             │
│   Like a restaurant waiter:                                 │
│   • You call them over (you start the program)              │
│   • They serve you (program does its task)                  │
│   • They leave when you're done (program exits)             │
│                                                             │
│                                                             │
│   DAEMON:                                                   │
│                                                             │
│   Like a security guard:                                    │
│   • Shows up when the building opens (system boots)         │
│   • Sits quietly watching (runs in background)              │
│   • Reacts when something happens (handles events)          │
│   • Never leaves until the building closes (system shuts    │
│     down)                                                   │
│   • You don't interact with them directly                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Regular Program vs Daemon

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   REGULAR PROGRAM                      DAEMON                       │
│                                                                     │
│   ┌───────────────────┐               ┌───────────────────┐         │
│   │                   │               │                   │         │
│   │  $ ./my-program   │               │  Started by       │         │
│   │                   │               │  systemd on login │         │
│   │  (runs in your    │               │                   │         │
│   │   terminal)       │               │  (no terminal)    │         │
│   │                   │               │  (no user input)  │         │
│   │  User sees output │               │  (invisible)      │         │
│   │  User can type    │               │                   │         │
│   │                   │               │  Logs go to file  │         │
│   │  Ctrl+C to stop   │               │  or journald      │         │
│   │                   │               │                   │         │
│   │  Dies when        │               │  Survives terminal│         │
│   │  terminal closes  │               │  closing, logout  │         │
│   │                   │               │                   │         │
│   └───────────────────┘               └───────────────────┘         │
│                                                                     │
│   Lifecycle:                           Lifecycle:                    │
│   start → run → exit                  start → run forever           │
│   (minutes)                            (days/weeks/months)          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Common Linux Daemons

You already use dozens of daemons every day without knowing:

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Daemons On Your System                          │
│                                                                     │
│   Name                What It Does                                  │
│   ─────────────────   ──────────────────────────────────────        │
│   sshd                Listens for SSH connections                   │
│   NetworkManager      Manages WiFi and network connections          │
│   bluetoothd          Manages Bluetooth devices                     │
│   pipewire            Handles audio input/output                    │
│   cupsd               Manages printers                              │
│   crond               Runs scheduled tasks                          │
│   udevd               Detects hardware changes (USB plugged in)     │
│   Xorg                The X display server (yes, it's a daemon!)    │
│                                                                     │
│   Notice the "d" suffix? It stands for "daemon".                    │
│   sshd = SSH daemon                                                 │
│   bluetoothd = Bluetooth daemon                                     │
│                                                                     │
│   shake-cursor will be another daemon in this list.                 │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## How Daemons Are Managed: systemd

On modern Linux (including Fedora), **systemd** is the program that manages all daemons. It:

- **Starts** daemons when the system boots or user logs in
- **Stops** daemons when the system shuts down
- **Restarts** daemons if they crash
- **Monitors** daemons (tracks CPU, memory usage)
- **Collects logs** from daemons

```
┌─────────────────────────────────────────────────────────────────────┐
│                          systemd                                    │
│                    (the daemon manager)                              │
│                                                                     │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                                                             │   │
│   │   systemd                                                   │   │
│   │     │                                                       │   │
│   │     ├── sshd           (running, pid 1234)                  │   │
│   │     ├── NetworkManager (running, pid 1235)                  │   │
│   │     ├── bluetoothd     (running, pid 1236)                  │   │
│   │     ├── pipewire       (running, pid 1237)                  │   │
│   │     ├── Xorg           (running, pid 1238)                  │   │
│   │     └── shake-cursor   (running, pid 1239)  ← our daemon   │   │
│   │                                                             │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                     │
│   systemd watches all of them. If one crashes, it restarts it.      │
│   If the system shuts down, it stops them all gracefully.           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## System Daemons vs User Daemons

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│   SYSTEM DAEMONS                       USER DAEMONS                 │
│   (run as root)                        (run as your user)           │
│                                                                     │
│   Start: on boot                       Start: on login              │
│   Stop:  on shutdown                   Stop:  on logout             │
│   User:  root                          User:  your account          │
│   Config: /etc/systemd/system/         Config: ~/.config/systemd/   │
│                                                user/                │
│                                                                     │
│   Examples:                            Examples:                    │
│   • sshd                               • pipewire                  │
│   • NetworkManager                     • gpg-agent                  │
│   • udevd                              • shake-cursor ← this one   │
│                                                                     │
│   Need root access to hardware         Only need user-level access  │
│   and system-wide resources            and user's own resources     │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

**shake-cursor is a user daemon** because:
- It only needs access to the X server (which your user owns)
- It doesn't need root privileges
- It should start when YOU log in, not when the system boots
- It only makes sense when there's a graphical session

## Service Files: How to Define a Daemon

A **service file** tells systemd how to manage your daemon:

```
File: ~/.config/systemd/user/shake-cursor.service

┌─────────────────────────────────────────────────────────────┐
│  [Unit]                                                     │
│  Description=Shake cursor to enlarge it                     │
│  After=graphical-session.target                             │
│                                                             │
│  [Service]                                                  │
│  ExecStart=/usr/local/bin/shake-cursor                      │
│  Restart=on-failure                                         │
│  RestartSec=3                                               │
│                                                             │
│  [Install]                                                  │
│  WantedBy=default.target                                    │
└─────────────────────────────────────────────────────────────┘

What each section means:

[Unit]
Description  → Human-readable name
After        → Start after the graphical session is ready

[Service]
ExecStart    → The command to run
Restart      → Restart if it crashes
RestartSec   → Wait 3 seconds before restarting

[Install]
WantedBy     → Start when user session begins
```

## Managing a Daemon with systemctl

```
┌─────────────────────────────────────────────────────────────────────┐
│                     systemctl Commands                               │
│                                                                     │
│   ACTION              COMMAND                                       │
│   ─────────────────   ──────────────────────────────────            │
│                                                                     │
│   Enable (auto-start  systemctl --user enable shake-cursor          │
│   on login)                                                         │
│                                                                     │
│   Start now           systemctl --user start shake-cursor           │
│                                                                     │
│   Enable + Start      systemctl --user enable --now shake-cursor    │
│                                                                     │
│   Stop                systemctl --user stop shake-cursor            │
│                                                                     │
│   Disable             systemctl --user disable shake-cursor         │
│   (no auto-start)                                                   │
│                                                                     │
│   Check status        systemctl --user status shake-cursor          │
│                                                                     │
│   View logs           journalctl --user -u shake-cursor             │
│                                                                     │
│   View live logs      journalctl --user -u shake-cursor -f          │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## Daemon Lifecycle for shake-cursor

```
┌────────────────────────────────────────────────────────────────────┐
│                                                                    │
│   1. USER LOGS IN                                                  │
│      │                                                             │
│      ▼                                                             │
│   2. systemd starts graphical session (Xorg + XFCE)               │
│      │                                                             │
│      ▼                                                             │
│   3. systemd starts shake-cursor                                   │
│      │                                                             │
│      ▼                                                             │
│   4. shake-cursor connects to X server                             │
│      │                                                             │
│      ▼                                                             │
│   5. shake-cursor subscribes to MotionNotify events                │
│      │                                                             │
│      ▼                                                             │
│   6. EVENT LOOP (runs forever)                                     │
│      │                                                             │
│      │   ┌──────────────────────────────────────────┐              │
│      │   │  sleep... (0% CPU)                       │              │
│      │   │       │                                  │              │
│      │   │       ▼                                  │              │
│      │   │  MotionNotify! → analyze movement        │              │
│      │   │       │                                  │              │
│      │   │       ├── normal → do nothing            │              │
│      │   │       │                                  │              │
│      │   │       └── shake! → enlarge cursor        │              │
│      │   │                                          │              │
│      │   │  sleep... (0% CPU)                       │              │
│      │   │       │                                  │              │
│      │   └───────┘ (repeat forever)                 │              │
│      │                                                             │
│   7. USER LOGS OUT                                                 │
│      │                                                             │
│      ▼                                                             │
│   8. systemd sends SIGTERM to shake-cursor                         │
│      │                                                             │
│      ▼                                                             │
│   9. shake-cursor disconnects from X server and exits              │
│                                                                    │
│                                                                    │
│   IF shake-cursor CRASHES:                                         │
│      systemd detects → waits 3 seconds → restarts it               │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

## Summary

| Concept | Definition |
|---------|------------|
| **Daemon** | A background process that runs continuously without user interaction |
| **systemd** | The program that manages daemons (starts, stops, restarts, monitors) |
| **Service file** | Configuration that tells systemd how to run your daemon |
| **User daemon** | A daemon that runs per-user (not system-wide), starts on login |
| **SIGTERM** | The signal systemd sends to a daemon to ask it to shut down |
| **journald** | Where daemon logs go (viewed with `journalctl`) |

shake-cursor will be a **user daemon** managed by systemd that:
1. Starts automatically when you log in
2. Connects to the X server
3. Listens for mouse motion events (sleeping when idle)
4. Detects shake patterns and enlarges the cursor
5. Shuts down cleanly when you log out
6. Auto-restarts if it crashes
