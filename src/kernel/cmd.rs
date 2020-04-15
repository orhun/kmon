use crate::style::Symbol;

/* Kernel module related command */
#[derive(Debug)]
pub struct Command {
	pub cmd: String,
	pub desc: &'static str,
	pub title: String,
	pub symbol: Symbol,
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
		symbol: Symbol,
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
	Reload,
	Blacklist,
	Clear,
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
            Self::None => Command::new(String::from(""), "", format!("Module: {}", module_name), Symbol::None),
            Self::Load => Command::new(
				format!("modprobe {}", &module_name),
				"modprobe: Add and remove modules from the Linux Kernel\n
				This command inserts a module to the kernel.",
				format!("Load: {}", module_name), Symbol::Anchor),
            Self::Unload => Command::new(
                format!("modprobe -r {}", &module_name),
                "modprobe: Add and remove modules from the Linux Kernel
                option: -r, --remove\n
                This option causes modprobe to remove rather than insert a module. \
                If the modules it depends on are also unused, modprobe will try to \
				remove them too.
                There is usually no reason to remove modules, but some buggy \
                modules require it. Your distribution kernel may not have been \
                built to support removal of modules at all.",
                format!("Remove: {}", module_name), Symbol::CircleX),
            Self::Reload => Command::new(
                format!("{} && {}",
                    ModuleCommand::Unload.get(module_name).cmd,
                    ModuleCommand::Load.get(module_name).cmd),
                "modprobe: Add and remove modules from the Linux Kernel\n
                This command reloads a module, removes and inserts to the kernel.",
                format!("Reload: {}", module_name), Symbol::FuelPump),
			Self::Blacklist => Command::new(
				format!("if ! grep -q {module} /etc/modprobe.d/blacklist.conf; then
				  echo 'blacklist {module}' >> /etc/modprobe.d/blacklist.conf
				  echo 'install {module} /bin/false' >> /etc/modprobe.d/blacklist.conf
				fi", module = &module_name),
				"This command blacklists a module and any other module that depends on it.\n
				Blacklisting is a mechanism to prevent the kernel module from loading. \
				This could be useful if, for example, the associated hardware is not needed, \
				or if loading that module causes problems.
				The blacklist command will blacklist a module so that it will not be loaded \
				automatically, but the module may be loaded if another non-blacklisted module \
				depends on it or if it is loaded manually. However, there is a workaround for \
				this behaviour; the install command instructs modprobe to run a custom command \
				instead of inserting the module in the kernel as normal, so the module will \
				always fail to load.",
				format!("Blacklist: {}", module_name), Symbol::SquareX),
			Self::Clear => Command::new(
				String::from("dmesg --clear"),
				"dmesg: Print or control the kernel ring buffer
				option: -C, --clear\n
				Clear the ring buffer.",
				String::from("Clear"), Symbol::Cloud),
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
		assert_eq!(
			format!(
				"{} && {}",
				ModuleCommand::Unload.get("x").cmd,
				ModuleCommand::Load.get("x").cmd
			),
			ModuleCommand::Reload.get("x").cmd,
		);
	}
}
