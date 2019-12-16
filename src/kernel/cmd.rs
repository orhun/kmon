/* Kernel module related command */
pub struct Command<'a> {
	pub cmd: String,
	pub desc: &'a str,
	pub title: String,
}

impl Command<'_> {
	/**
	 * Create a new command instance.
	 *
	 * @param  command
	 * @param  description
	 * @param  command_title
	 * @return Command
	 */
	fn new(
		command: String,
		description: &'static str,
		mut command_title: String,
	) -> Self {
		/* Parse the command title if '!' is given. */
		if command_title.contains('!) {
			command_title = command_title
				.split('!')
				.collect::<Vec<&str>>()
				.last()
				.unwrap_or(&"")
				.to_string();
		}
		Self {
			cmd: command,
			desc: description,
			title: command_title,
		}
	}
}

/* Kernel module management commands */
#[derive(PartialEq)]
pub enum ModuleCommand {
	None,
	Load,
	Unload,
}

impl ModuleCommand {
	/**
	 * Get Command struct from a enum element.
	 *
	 * @param  module_name
	 * @return Command
	 */
	pub fn get(&self, module_name: &str) -> Command {
		match self {
            Self::None => Command::new(String::from(""), "", format!("Module: {}", module_name)),
            Self::Load => Command::new(
				format!("modprobe {}", &module_name),
				"modprobe: Add and remove modules from the Linux Kernel\n
                                This command inserts a module to the kernel.",
				format!("Load: {}", module_name)),
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
                format!("Remove: {}", module_name),
            ),
        }
	}
	/**
	 * Check if module command is set.
	 *
	 * @return bool
	 */
	pub fn is_none(&self) -> bool {
		self == &Self::None
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
	}
}
