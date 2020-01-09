pub mod cmd;
pub mod info;
pub mod lkm;
pub mod log;
use clap::ArgMatches;
use info::KernelInfo;
use lkm::KernelModules;
use log::KernelLogs;

/* Kernel struct for logs, information and modules */
pub struct Kernel {
	pub logs: KernelLogs,
	pub info: KernelInfo,
	pub modules: KernelModules<'static>,
}

impl Kernel {
	pub fn new(args: &ArgMatches) -> Self {
		Self {
			logs: KernelLogs::default(),
			info: KernelInfo::new(),
			modules: KernelModules::new(args),
		}
	}
}
