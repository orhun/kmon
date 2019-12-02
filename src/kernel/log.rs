use crate::util::exec_cmd;

/* Kernel logs struct and implementation. */
pub struct KernelLogs {
    pub output: String,
    last_line: String,
    pub scroll_offset: u16,
}
impl KernelLogs {
    /**
     * Create a new kernel logs instance.
     *
     * @return KernelLogs
     */
    pub fn new() -> Self {
        Self {
            output: String::new(),
            last_line: String::new(),
            scroll_offset: 0,
        }
    }
    /**
     * Update the output variable value if 'dmesg' logs changed.
     *
     * @return logs_updated
     */
    pub fn update(&mut self) -> bool {
        self.output = exec_cmd("sh", &["-c", "dmesg --kernel --human --color=never | tac"])
            .expect("failed to retrieve dmesg output");
        let logs_updated = self.output.lines().next().unwrap() != &self.last_line;
        self.last_line = self.output.lines().next().unwrap().to_string();
        logs_updated
    }
}
