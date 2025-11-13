use std::io::{stdout, Write};
use std::process;

use actix_files::Files;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, middleware, Responder, web};
use seccompiler::{BpfProgram, SeccompAction, SeccompCmpArgLen, SeccompCmpOp, SeccompCondition, SeccompFilter, SeccompRule, TargetArch};

async fn index(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

async fn robots(_req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../res/robots.txt"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var("RUST_LOG").is_ok() {
        println!("WARNING: Logging is enabled!");
        env_logger::init();
    } else {
        println!("For logging set RUST_LOG to actix_server=debug,actix_web=debug")
    }

    let seccomp_filter = SeccompFilter::new(
        vec![
            // these syscalls are always allowed with all arguments
            (libc::SYS_uname, vec![]),
            (libc::SYS_getcwd, vec![]),
            (libc::SYS_close, vec![]),
            (libc::SYS_epoll_wait, vec![]),
            (libc::SYS_futex, vec![]),
            (libc::SYS_epoll_ctl, vec![]),
            (libc::SYS_munmap, vec![]),
            (libc::SYS_getrandom, vec![]),
            (libc::SYS_exit, vec![]),
            (libc::SYS_exit_group, vec![]),
            (libc::SYS_read, vec![]),
            (libc::SYS_write, vec![]),
            (libc::SYS_fstat, vec![]),
            (libc::SYS_faccessat2, vec![]),

            // TODO: i dont know if these syscalls are dangerous or if they need to be restricted
            (libc::SYS_rt_sigprocmask, vec![]),
            (libc::SYS_rt_sigaction, vec![]),
            (libc::SYS_rt_sigreturn, vec![]),
            (libc::SYS_sigaltstack, vec![]),
            (libc::SYS_getsockname, vec![]),
            (libc::SYS_getpeername, vec![]),
            (libc::SYS_set_robust_list, vec![]),
            (libc::SYS_eventfd2, vec![]),
            (libc::SYS_rseq, vec![]),
            (libc::SYS_listen, vec![]),
            (libc::SYS_bind, vec![]),
            (libc::SYS_recvmsg, vec![]),
            (libc::SYS_connect, vec![]),
            (libc::SYS_shutdown, vec![]),
            (libc::SYS_sched_getaffinity, vec![]),
            (libc::SYS_statx, vec![]),
            (libc::SYS_readlink, vec![]), // always return -1 EINVAL
            (libc::SYS_newfstatat, vec![]),

            // these syscalls are restricted
            (libc::SYS_openat,
             vec![
                 // fd only AT_FDCWD always as O_RDONLY and O_CLOEXEC
                 SeccompRule::new(vec![
                     SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::AT_FDCWD as u64).unwrap(),
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, (libc::O_RDONLY | libc::O_CLOEXEC) as u64).unwrap(),
                 ]).unwrap()
             ]),
            (libc::SYS_lseek,
             vec![
                 // second argument (offset) is always 0
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, 0).unwrap()
                 ]).unwrap()
             ]),
            (libc::SYS_mmap,
             vec![
                 // may only be MAP_PRIVATE and may only be PROT_NONE, PROT_READ, or PROT_WRITE ored
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::PROT_NONE as u64), libc::PROT_NONE as u64).unwrap(),
                     SeccompCondition::new(3, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::MAP_PRIVATE as u64), libc::MAP_PRIVATE as u64).unwrap(),
                 ]).unwrap(),
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::PROT_READ as u64), libc::PROT_READ as u64).unwrap(),
                     SeccompCondition::new(3, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::MAP_PRIVATE as u64), libc::MAP_PRIVATE as u64).unwrap(),
                 ]).unwrap(),
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::PROT_WRITE as u64), libc::PROT_WRITE as u64).unwrap(),
                     SeccompCondition::new(3, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::MAP_PRIVATE as u64), libc::MAP_PRIVATE as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_mprotect,
             vec![
                 // may only be PROT_NONE, PROT_READ, or PROT_WRITE ored
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::PROT_NONE as u64), libc::PROT_NONE as u64).unwrap(),
                 ]).unwrap(),
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::PROT_READ as u64), libc::PROT_READ as u64).unwrap(),
                 ]).unwrap(),
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::PROT_WRITE as u64), libc::PROT_WRITE as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_fcntl,
             vec![
                 // second arg (cmd) may only be F_DUPFD_CLOEXEC or F_GETFD or F_GETFL
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::F_DUPFD_CLOEXEC as u64).unwrap(),
                 ]).unwrap(),
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::F_GETFD as u64).unwrap(),
                 ]).unwrap(),
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::F_GETFL as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_prctl,
             vec![
                 // only allow PR_SET_NAME
                 SeccompRule::new(vec![
                     SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::PR_SET_NAME as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_clone3,
             vec![
                 // only allow CLONE_VM|CLONE_FS|CLONE_FILES|CLONE_SIGHAND|CLONE_THREAD|CLONE_SYSVSEM|CLONE_SETTLS|CLONE_PARENT_SETTID|CLONE_CHILD_CLEARTID,
                 // not possible with this API :(
             ]),
            (libc::SYS_accept4,
             vec![
                 // only allowed flags are SOCK_CLOEXEC|SOCK_NONBLOCK
                 SeccompRule::new(vec![
                     SeccompCondition::new(3, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, (libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK) as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_recvfrom,
             vec![
                 // last three args must be 0
                 SeccompRule::new(vec![
                     SeccompCondition::new(3, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, 0).unwrap(),
                     SeccompCondition::new(4, SeccompCmpArgLen::Qword, SeccompCmpOp::Eq, 0).unwrap(),
                     SeccompCondition::new(5, SeccompCmpArgLen::Qword, SeccompCmpOp::Eq, 0).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_sendto,
             vec![
                 // last two args must be 0 and flags must be MSG_NOSIGNAL
                 // interferes with netlink sendto
                 /*SeccompRule::new(vec![
                     //SeccompCondition::new(3, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::MSG_NOSIGNAL as u64).unwrap(),
                     //SeccompCondition::new(4, SeccompCmpArgLen::Qword, SeccompCmpOp::Eq, 0).unwrap(),
                     //SeccompCondition::new(5, SeccompCmpArgLen::Qword, SeccompCmpOp::Eq, 0).unwrap(),
                 ]).unwrap(),*/
             ]),
            (libc::SYS_epoll_create1,
             vec![
                 // only allowed with flag EPOLL_CLOEXEC
                 SeccompRule::new(vec![
                     SeccompCondition::new(0, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::EPOLL_CLOEXEC as u64).unwrap()
                 ]).unwrap(),
             ]),
            (libc::SYS_socket,
             vec![
                 // flags must contain SOCK_CLOEXEC
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::MaskedEq(libc::SOCK_CLOEXEC as u64), libc::SOCK_CLOEXEC as u64).unwrap()
                 ]).unwrap(),
             ]),
            (libc::SYS_setsockopt,
             vec![
                 // only allow SOL_SOCKET and SO_REUSEADDR
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::SOL_SOCKET as u64).unwrap(),
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::SO_REUSEADDR as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_ioctl,
             vec![
                 // only allow FIONBIO
                 SeccompRule::new(vec![
                     SeccompCondition::new(1, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::FIONBIO as u64).unwrap(),
                 ]).unwrap(),
             ]),
            (libc::SYS_madvise,
             vec![
                 // advice may only be MADV_DONTNEED
                 SeccompRule::new(vec![
                     SeccompCondition::new(2, SeccompCmpArgLen::Dword, SeccompCmpOp::Eq, libc::MADV_DONTNEED as u64).unwrap(),
                 ]).unwrap(),
             ]),
        ].into_iter().collect(),
        SeccompAction::KillProcess,
        SeccompAction::Allow,
        TargetArch::x86_64,
    ).unwrap();

    let bpf_program: BpfProgram = seccomp_filter.try_into().unwrap();

    println!("Going into lockdown mode pid = {}", process::id());
    stdout().lock().flush().unwrap();
    seccompiler::apply_filter_all_threads(&bpf_program).unwrap();
    println!("Applied filter");
    println!("Starting server at http://localhost:8001");

    if std::env::var("RUST_LOG").is_ok() {
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Logger::default())
                .wrap(middleware::Compress::default())
                .service(web::resource("/robots.txt").route(web::get().to(robots)))
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/static", "static").prefer_utf8(true))
        })
            .bind("localhost:8001")?
            .run()
            .await
    } else {
        HttpServer::new(|| {
            App::new()
                .wrap(middleware::Compress::default())
                .service(web::resource("/robots.txt").route(web::get().to(robots)))
                .service(web::resource("/").route(web::get().to(index)))
                .service(Files::new("/static", "static").prefer_utf8(true))
        })
            .bind("localhost:8001")?
            .run()
            .await
    }

    // needed syscalls:
    // * getcwd
    // * readlink, openat, statx only in static folder
}
