struct Plic {
    base: usize,
    max_priority: usize,
    max_target: usize,
    target: usize,
    threshold: usize,
    claim: usize,
    enable: [usize; 2],
    priority: [usize; 2],
}
impl Plic {
    pub fn new(base: usize) -> Self {
        let max_priority = 0x4;
        let max_target = 0x3;
        Self {
            base,
            max_priority,
            max_target,
            target: base + 0x2000,
            threshold: base + 0x200000,
            claim: base + 0x200004,
            enable: [base + 0x2080, base + 0x2084],
            priority: [base + 0x200000, base + 0x200004],
        }
    }
    pub fn set_threshold(&mut self, target: usize, threshold: usize) {
        unsafe {
            (self.target + target * 4).write_volatile(threshold);
        }
    }
    pub fn set_enable(&mut self, irq: usize, enable: bool) {
        let index = irq / 32;
        let shift = irq % 32;
        unsafe {
            if enable {
                (self.enable[index] + index * 4).write_volatile(1 << shift);
            } else {
                (self.enable[index] + index * 4).write_volatile(!(1 << shift));
            }
        }
    }
    pub fn set_priority(&mut self, irq: usize, priority: usize) {
        unsafe {
            (self.priority[irq / 32] + irq * 4).write_volatile(priority);
        }
    }
    pub fn claim(&mut self) -> usize {
        unsafe { self.claim.read_volatile() }
    }
    pub fn complete(&mut self, irq: usize) {
        unsafe {
            self.claim.write_volatile(irq);
        }
    }
}
struct Vplic {
    plic: Plic,
    vcpu: usize,
}
impl Vplic {
    pub fn new(base: usize, vcpu: usize) -> Self {
        Self {
            plic: Plic::new(base),
            vcpu,
        }
    }
    pub fn set_threshold(&mut self, target: usize, threshold: usize) {
        self.plic.set_threshold(target, threshold);
    }
    pub fn set_enable(&mut self, irq: usize, enable: bool) {
        self.plic.set_enable(irq, enable);
    }
    pub fn set_priority(&mut self, irq: usize, priority: usize) {
        self.plic.set_priority(irq, priority);
    }
    pub fn claim(&mut self) -> usize {
        self.plic.claim()
    }
    pub fn complete(&mut self, irq: usize) {
        self.plic.complete(irq);
    }
}
