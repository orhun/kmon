use crate::style::UnicodeSymbol;

/* Kernel module related command */
#[derive(Debug)]
pub struct Command {
	pub cmd: String,
	pub desc: &'static str,
	pub title: String,
	pub symbol: UnicodeSymbol,
}

impl Command {
	/**
	 * Create a new command instance.
	 *
	 * @param  command
	 * @param  description
	 * @param  command_title
	 * @return Command
	 */
	fn new(
		cmd: String,
		desc: &'static str,
		mut title: String,
		symbol: UnicodeSymbol,
	) -> Self {
		/* Parse the command title if '!' is given. */
		if title.contains('!') {
			title = (*title
				.split('!')
				.collect::<Vec<&str>>()
				.last()
				.unwrap_or(&""))
			.to_string();
		}
		Self {
			cmd,
			desc,
			title,
			symbol,
		}
	}
}

/* Kernel module management commands */
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ModuleCommand {
	None,
	Load,
	Unload,
	Blacklist,
}

impl ModuleCommand {
	/**
	 * Get Command struct from a enum element.
	 *
	 * @param  module_name
	 * @return Command
	 */
	pub fn get(self, module_name: &str) -> Command {
		match self {
            Self::None => Command::new(String::from(""), "", format!("Module: {}", module_name), UnicodeSymbol::None),
            Self::Load => Command::new(
				format!("modprobe {}", &module_name),
				"modprobe: Add and remove modules from the Linux Kernel\n
                                This command inserts a module to the kernel.",
				format!("Load: {}", module_name), UnicodeSymbol::Anchor),
            Self::Unload => Command::new(
                format!("modprobe -r {}", &module_name),
                "modprobe: Add and remove modules from the Linux Kernel
                option:   -r, --remove\n
                This option causes modprobe to remove rather than insert a module. \
                If the modules it depends on are also unused, modprobe will try to \
				remove them too. Unlike insertion, more than one module can be \
                specified on the command line (it does not make sense to specify \
                module parameters when removing modules).\n
                There is usually no reason to remove modules, but some buggy \
                modules require it. Your distribution kernel may not have been \
                built to support removal of modules at all.",
                format!("Remove: {}", module_name), UnicodeSymbol::CircleX,
            ),
			Self::Blacklist => Command::new(
				format!("echo 'blacklist {}' >> /etc/modprobe.d/blacklist.conf", &module_name),
				"Blacklisting is a mechanism to prevent the kernel module from loading. \
				This could be useful if, for example, the associated hardware is not needed, \
				or if loading that module causes problems. For instance, there may be two \
				kernel modules that try to control the same piece of hardware, and loading \
				them together would result in a conflict.\n
				You might want to regenerate the initial ramdisk image and reboot after \
				blacklisting the modules depending on your configuration.",
				format!("Blacklist: {}", module_name), UnicodeSymbol::SquareX),
        }
	}

	/**
	 * Check if module command is set.
	 *
	 * @return bool
	 */
	pub fn is_none(self) -> bool {
		self == Self::None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn test_module_command() {
		let module_command = ModuleCommand::None;
		assert_eq!(true, module_command == ModuleCommand::None);
		assert_ne!("", ModuleCommand::None.get("test").title);
		assert_ne!("", ModuleCommand::Load.get("module").desc);
		assert_ne!("", ModuleCommand::Unload.get("!command").cmd);
		assert_ne!("", ModuleCommand::Blacklist.get("~").cmd);
	}
}
