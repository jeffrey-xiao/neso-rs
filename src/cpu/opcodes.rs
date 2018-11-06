use cpu::{addressing_modes, registers, Cpu, Interrupt};

pub struct Operand {
    pub val: u8,
    pub addr: Option<u16>,
    pub page_crossing: bool,
}

#[rustfmt::skip]
pub const INSTRUCTION_TABLE: [fn(&mut Cpu, usize) -> (); 256] = [
    brk, ora, inv, slo, dop, ora, asl, slo, php, ora, asl, anc, top, ora, asl, slo, // 00
    bpl, ora, inv, slo, dop, ora, asl, slo, clc, ora, nop, slo, top, ora, asl, slo, // 10
    jsr, and, inv, rla, bit, and, rol, rla, plp, and, rol, anc, bit, and, rol, rla, // 20
    bmi, and, inv, rla, dop, and, rol, rla, sec, and, nop, rla, top, and, rol, rla, // 30
    rti, eor, inv, sre, dop, eor, lsr, sre, pha, eor, lsr, asr, jmp, eor, lsr, sre, // 40
    bvc, eor, inv, sre, dop, eor, lsr, sre, cli, eor, nop, sre, top, eor, lsr, sre, // 50
    rts, adc, inv, rra, dop, adc, ror, rra, pla, adc, ror, arr, jmp, adc, ror, rra, // 60
    bvs, adc, inv, rra, dop, adc, ror, rra, sei, adc, nop, rra, top, adc, ror, rra, // 70
    dop, sta, dop, aax, sty, sta, stx, aax, dey, dop, txa, xaa, sty, sta, stx, aax, // 80
    bcc, sta, inv, axa, sty, sta, stx, aax, tya, sta, txs, tas, shy, sta, shx, axa, // 90
    ldy, lda, ldx, lax, ldy, lda, ldx, lax, tay, lda, tax, lax, ldy, lda, ldx, lax, // A0
    bcs, lda, inv, lax, ldy, lda, ldx, lax, clv, lda, tsx, las, ldy, lda, ldx, lax, // B0
    cpy, cmp, dop, dcp, cpy, cmp, dec, dcp, iny, cmp, dex, axs, cpy, cmp, dec, dcp, // C0
    bne, cmp, inv, dcp, dop, cmp, dec, dcp, cld, cmp, nop, dcp, top, cmp, dec, dcp, // D0
    cpx, sbc, dop, isc, cpx, sbc, inc, isc, inx, sbc, nop, sbc, cpx, sbc, inc, isc, // E0
    beq, sbc, inv, isc, dop, sbc, inc, isc, sed, sbc, nop, isc, top, sbc, inc, isc, // F0
];

#[rustfmt::skip]
pub const CYCLE_TABLE: [u8; 256] = [
    7, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 4, 4, 6, 6, // 00
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 10
    6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 4, 4, 6, 6, // 20
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 30
    6, 6, 0, 8, 3, 3, 5, 5, 3, 2, 2, 2, 3, 4, 6, 6, // 40
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 50
    6, 6, 0, 8, 3, 3, 5, 5, 4, 2, 2, 2, 5, 4, 6, 6, // 60
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // 70
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, // 80
    2, 6, 0, 6, 4, 4, 4, 4, 2, 5, 2, 5, 5, 5, 5, 5, // 90
    2, 6, 2, 6, 3, 3, 3, 3, 2, 2, 2, 2, 4, 4, 4, 4, // A0
    2, 5, 0, 5, 4, 4, 4, 4, 2, 4, 2, 4, 4, 4, 4, 4, // B0
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, // C0
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // D0
    2, 6, 2, 8, 3, 3, 5, 5, 2, 2, 2, 2, 4, 4, 6, 6, // E0
    2, 5, 0, 8, 4, 4, 6, 6, 2, 4, 2, 7, 4, 4, 7, 7, // F0
];

#[rustfmt::skip]
pub const ADDRESSING_MODE_TABLE: [usize; 256] = [
     6,  8,  0,  8, 11, 11, 11, 11,  6,  5,  4,  5,  1,  1,  1,  1, // 00
    10,  9,  0,  9, 12, 12, 12, 12,  6,  3,  6,  3,  2,  2,  2,  2, // 10
     1,  8,  0,  8, 11, 11, 11, 11,  6,  5,  4,  5,  1,  1,  1,  1, // 20
    10,  9,  0,  9, 12, 12, 12, 12,  6,  3,  6,  3,  2,  2,  2,  2, // 30
     6,  8,  0,  8, 11, 11, 11, 11,  6,  5,  4,  5,  1,  1,  1,  1, // 40
    10,  9,  0,  9, 12, 12, 12, 12,  6,  3,  6,  3,  2,  2,  2,  2, // 50
     6,  8,  0,  8, 11, 11, 11, 11,  6,  5,  4,  5,  7,  1,  1,  1, // 60
    10,  9,  0,  9, 12, 12, 12, 12,  6,  3,  6,  3,  2,  2,  2,  2, // 70
     5,  8,  5,  8, 11, 11, 11, 11,  6,  5,  6,  5,  1,  1,  1,  1, // 80
    10,  9,  0,  9, 12, 12, 13, 13,  6,  3,  6,  3,  2,  2,  3,  3, // 90
     5,  8,  5,  8, 11, 11, 11, 11,  6,  5,  6,  5,  1,  1,  1,  1, // A0
    10,  9,  0,  9, 12, 12, 13, 13,  6,  3,  6,  3,  2,  2,  3,  3, // B0
     5,  8,  5,  8, 11, 11, 11, 11,  6,  5,  6,  5,  1,  1,  1,  1, // C0
    10,  9,  0,  9, 12, 12, 12, 12,  6,  3,  6,  3,  2,  2,  2,  2, // D0
     5,  8,  5,  8, 11, 11, 11, 11,  6,  5,  6,  5,  1,  1,  1,  1, // E0
    10,  9,  0,  9, 12, 12, 12, 12,  6,  3,  6,  3,  2,  2,  2,  2, // F0
];

fn inv(cpu: &mut Cpu, _addressing_mode: usize) {
    let addr = cpu.r.pc - 1;
    panic!("[CPU] Invalid opcode: {:#04x}.", cpu.read_byte(addr));
}

fn aax(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);

    let res = cpu.r.x & cpu.r.a;
    cpu.write_byte(addr, res);
}

fn adc_impl(cpu: &mut Cpu, operand: &Operand) {
    let carry = if cpu.r.p & registers::CARRY_MASK == 0 {
        0
    } else {
        1
    };
    let (res, is_overflow_1) = cpu.r.a.overflowing_add(operand.val);
    let (res, is_overflow_2) = res.overflowing_add(carry);
    let overflow = !(operand.val ^ cpu.r.a) & (res ^ cpu.r.a) & 0x80 != 0;
    cpu.r.update_nz_flags(res);
    cpu.r
        .set_status_flag(registers::CARRY_MASK, is_overflow_1 | is_overflow_2);
    cpu.r.set_status_flag(registers::OVERFLOW_MASK, overflow);
    cpu.r.a = res;
}

fn adc(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    adc_impl(cpu, &operand);
}

fn anc(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);

    and_impl(cpu, &operand);
    let res = cpu.r.a;
    cpu.r
        .set_status_flag(registers::CARRY_MASK, res & 0x80 != 0);
}

fn and_impl(cpu: &mut Cpu, operand: &Operand) {
    cpu.r.a &= operand.val;
    let res = cpu.r.a;
    cpu.r.update_nz_flags(res);
}

fn and(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    and_impl(cpu, &operand);
}

fn arr(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    and_impl(cpu, &operand);
    operand = cpu.get_operand(addressing_modes::ACCUMULATOR);
    let mut res = operand.val >> 1;
    res |= if cpu.r.get_status_flag(registers::CARRY_MASK) {
        0x80
    } else {
        0
    };
    cpu.r.update_nz_flags(res);
    let carry_bit = res & 0x40 != 0;
    let overflow_bit = carry_bit ^ (res & 0x20 != 0);
    cpu.r.set_status_flag(registers::CARRY_MASK, carry_bit);
    cpu.r
        .set_status_flag(registers::OVERFLOW_MASK, overflow_bit);
    operand.val = res;
    cpu.write_operand(&operand);
}

fn asl_impl(cpu: &mut Cpu, operand: &mut Operand) {
    let res = operand.val << 1;
    cpu.r.update_nz_flags(res);
    cpu.r
        .set_status_flag(registers::CARRY_MASK, operand.val & 0x80 != 0);

    operand.val = res;
    cpu.write_operand(operand);
}

fn asl(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    asl_impl(cpu, &mut operand);
}

fn asr(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);

    and_impl(cpu, &operand);
    lsr(cpu, addressing_modes::ACCUMULATOR);
}

fn axa(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_crossing) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let res = cpu.r.a & cpu.r.x & ((addr >> 8) as u8 + 1);
    cpu.write_byte(addr, res);
}

fn axs(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    let (res, underflow) = (cpu.r.a & cpu.r.x).overflowing_sub(operand.val);
    cpu.r.x = res;
    cpu.r.set_status_flag(registers::CARRY_MASK, !underflow);
    cpu.r.update_nz_flags(res);
}

fn branch_impl(cpu: &mut Cpu, cond: bool, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    if cond {
        cpu.cycle += 1;
        if cpu.r.pc & 0xFF00 != addr & 0xFF00 {
            cpu.cycle += 1;
        }
        cpu.r.pc = addr;
    }
}

fn bcc(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = !cpu.r.get_status_flag(registers::CARRY_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn bcs(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = cpu.r.get_status_flag(registers::CARRY_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn beq(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = cpu.r.get_status_flag(registers::ZERO_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn bit(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    cpu.r.set_status_flag(
        registers::NEGATIVE_MASK,
        operand.val & registers::NEGATIVE_MASK != 0,
    );
    cpu.r.set_status_flag(
        registers::OVERFLOW_MASK,
        operand.val & registers::OVERFLOW_MASK != 0,
    );

    let res = operand.val & cpu.r.a;
    cpu.r.update_zero_flag(res);
}

fn bmi(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = cpu.r.get_status_flag(registers::NEGATIVE_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn bne(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = !cpu.r.get_status_flag(registers::ZERO_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn bpl(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = !cpu.r.get_status_flag(registers::NEGATIVE_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn brk(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.pc += 1;
    cpu.handle_interrupt(Interrupt::IRQ as usize);
}

fn bvc(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = !cpu.r.get_status_flag(registers::OVERFLOW_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn bvs(cpu: &mut Cpu, addressing_mode: usize) {
    let cond = cpu.r.get_status_flag(registers::OVERFLOW_MASK);
    branch_impl(cpu, cond, addressing_mode);
}

fn clc(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.set_status_flag(registers::CARRY_MASK, false);
}

fn cld(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.set_status_flag(registers::DECIMAL_MODE_MASK, false);
}

fn cli(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r
        .set_status_flag(registers::INTERRUPT_DISABLE_MASK, false);
}

fn clv(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.set_status_flag(registers::OVERFLOW_MASK, false);
}

fn cmp_impl(cpu: &mut Cpu, operand: &Operand) {
    let (diff, underflow) = cpu.r.a.overflowing_sub(operand.val);
    cpu.r.set_status_flag(registers::CARRY_MASK, !underflow);
    cpu.r.update_nz_flags(diff);
}

fn cmp(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    cmp_impl(cpu, &operand);
}

fn cpx(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    let (diff, underflow) = cpu.r.x.overflowing_sub(operand.val);
    cpu.r.set_status_flag(registers::CARRY_MASK, !underflow);
    cpu.r.update_nz_flags(diff);
}

fn cpy(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    let (diff, underflow) = cpu.r.y.overflowing_sub(operand.val);
    cpu.r.set_status_flag(registers::CARRY_MASK, !underflow);
    cpu.r.update_nz_flags(diff);
}

fn dcp(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    dec_impl(cpu, &mut operand);
    cmp_impl(cpu, &operand);
}

fn dec_impl(cpu: &mut Cpu, operand: &mut Operand) {
    let res = operand.val.wrapping_sub(1);
    cpu.r.update_nz_flags(res);

    operand.val = res;
    cpu.write_operand(operand);
}

fn dec(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    dec_impl(cpu, &mut operand);
}

fn dex(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.x.wrapping_sub(1);
    cpu.r.update_nz_flags(res);
    cpu.r.x = res;
}

fn dey(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.y.wrapping_sub(1);
    cpu.r.update_nz_flags(res);
    cpu.r.y = res;
}

fn dop(cpu: &mut Cpu, addressing_mode: usize) {
    addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
}

fn eor_impl(cpu: &mut Cpu, operand: &Operand) {
    cpu.r.a ^= operand.val;
    let res = cpu.r.a;
    cpu.r.update_nz_flags(res);
}

fn eor(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    eor_impl(cpu, &operand);
}

fn inc_impl(cpu: &mut Cpu, operand: &mut Operand) {
    let res = operand.val.wrapping_add(1);
    cpu.r.update_nz_flags(res);

    operand.val = res;
    cpu.write_operand(operand);
}

fn inc(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    inc_impl(cpu, &mut operand);
}

fn inx(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.x.wrapping_add(1);
    cpu.r.update_nz_flags(res);
    cpu.r.x = res;
}

fn iny(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.y.wrapping_add(1);
    cpu.r.update_nz_flags(res);
    cpu.r.y = res;
}

fn isc(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    inc_impl(cpu, &mut operand);
    sbc_impl(cpu, &operand);
}

fn jmp(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    cpu.r.pc = addr;
}

fn jsr(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let ret = cpu.r.pc - 1;
    cpu.r.pc = addr;
    cpu.push_word(ret);
}

fn las(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    let res = operand.val & cpu.r.sp;
    cpu.r.a = res;
    cpu.r.x = res;
    cpu.r.sp = res;
    cpu.r.update_nz_flags(res);
}

fn lax(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    lda_impl(cpu, &operand);
    ldx_impl(cpu, &operand);
}

fn lda_impl(cpu: &mut Cpu, operand: &Operand) {
    cpu.r.a = operand.val;
    cpu.r.update_nz_flags(operand.val);
}

fn lda(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    lda_impl(cpu, &operand);
}

fn ldx_impl(cpu: &mut Cpu, operand: &Operand) {
    cpu.r.x = operand.val;
    cpu.r.update_nz_flags(operand.val);
}

fn ldx(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    ldx_impl(cpu, &operand);
}

fn ldy(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    cpu.r.y = operand.val;
    cpu.r.update_nz_flags(operand.val);
}

fn lsr_impl(cpu: &mut Cpu, operand: &mut Operand) {
    let res = operand.val >> 1;
    cpu.r.update_nz_flags(res);
    cpu.r
        .set_status_flag(registers::CARRY_MASK, operand.val & 0x01 != 0);

    operand.val = res;
    cpu.write_operand(operand);
}

fn lsr(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    lsr_impl(cpu, &mut operand);
}

fn nop(_cpu: &mut Cpu, _addressing_mode: usize) {}

fn ora_impl(cpu: &mut Cpu, operand: &Operand) {
    cpu.r.a |= operand.val;
    let res = cpu.r.a;
    cpu.r.update_nz_flags(res);
}

fn ora(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    ora_impl(cpu, &operand);
}

fn pha(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.a;
    cpu.push_byte(res);
}

fn php(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.p | 0x10;
    cpu.push_byte(res);
}

fn pla(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.pop_byte();
    cpu.r.a = res;
    cpu.r.update_nz_flags(res);
}

fn plp(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = (cpu.pop_byte() & !0x30) | (cpu.r.p & 0x30);
    cpu.r.p = res;
}

fn rla(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    rol_impl(cpu, &mut operand);
    and_impl(cpu, &operand);
}

fn rol_impl(cpu: &mut Cpu, operand: &mut Operand) {
    let mut res = operand.val << 1;
    res |= if cpu.r.get_status_flag(registers::CARRY_MASK) {
        1
    } else {
        0
    };
    cpu.r.update_nz_flags(res);
    cpu.r
        .set_status_flag(registers::CARRY_MASK, operand.val & 0x80 != 0);

    operand.val = res;
    cpu.write_operand(operand);
}

fn rol(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    rol_impl(cpu, &mut operand);
}

fn ror_impl(cpu: &mut Cpu, operand: &mut Operand) {
    let mut res = operand.val >> 1;
    res |= if cpu.r.get_status_flag(registers::CARRY_MASK) {
        0x80
    } else {
        0
    };
    cpu.r.update_nz_flags(res);
    cpu.r
        .set_status_flag(registers::CARRY_MASK, operand.val & 0x01 != 0);

    operand.val = res;
    cpu.write_operand(operand);
}

fn ror(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    ror_impl(cpu, &mut operand);
}

fn rti(cpu: &mut Cpu, _addressing_mode: usize) {
    plp(cpu, addressing_modes::IMPLIED);
    cpu.r.pc = cpu.pop_word();
    // println!("[CPU] Returning from interrupt.");
}

fn rts(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.pc = cpu.pop_word() + 1;
}

fn rra(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    ror_impl(cpu, &mut operand);
    adc_impl(cpu, &operand);
}

fn sbc_impl(cpu: &mut Cpu, operand: &Operand) {
    let carry = if cpu.r.p & registers::CARRY_MASK == 0 {
        1
    } else {
        0
    };
    let (res, is_underflow_1) = cpu.r.a.overflowing_sub(operand.val);
    let (res, is_underflow_2) = res.overflowing_sub(carry);
    let underflow = (operand.val ^ cpu.r.a) & (res ^ cpu.r.a) & 0x80 != 0;
    cpu.r.update_nz_flags(res);
    cpu.r
        .set_status_flag(registers::CARRY_MASK, !is_underflow_1 && !is_underflow_2);
    cpu.r.set_status_flag(registers::OVERFLOW_MASK, underflow);
    cpu.r.a = res;
}

fn sbc(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    if operand.page_crossing {
        cpu.cycle += 1;
    }

    sbc_impl(cpu, &operand);
}

fn sec(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.set_status_flag(registers::CARRY_MASK, true);
}

fn sed(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r.set_status_flag(registers::DECIMAL_MODE_MASK, true);
}

fn sei(cpu: &mut Cpu, _addressing_mode: usize) {
    cpu.r
        .set_status_flag(registers::INTERRUPT_DISABLE_MASK, true);
}

fn shx(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let res = cpu.r.x & ((addr >> 8) as u8 + 1);

    if !page_break {
        cpu.write_byte(addr, res);
    }
}

fn shy(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let res = cpu.r.y & ((addr >> 8) as u8 + 1);

    if !page_break {
        cpu.write_byte(addr, res);
    }
}

fn slo(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    asl_impl(cpu, &mut operand);
    ora_impl(cpu, &operand);
}

fn sta(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let res = cpu.r.a;
    cpu.write_byte(addr, res);
}

fn stx(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let res = cpu.r.x;
    cpu.write_byte(addr, res);
}

fn sty(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_break) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let res = cpu.r.y;
    cpu.write_byte(addr, res);
}

fn sre(cpu: &mut Cpu, addressing_mode: usize) {
    let mut operand = cpu.get_operand(addressing_mode);

    lsr_impl(cpu, &mut operand);
    eor_impl(cpu, &operand);
}

fn tas(cpu: &mut Cpu, addressing_mode: usize) {
    let (addr, _page_crossing) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    let mut res = cpu.r.a & cpu.r.x;
    cpu.r.sp = res;
    res &= (addr >> 8) as u8 + 1;
    cpu.write_byte(addr, res);
}

fn tax(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.a;
    cpu.r.update_nz_flags(res);
    cpu.r.x = res;
}

fn tay(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.a;
    cpu.r.update_nz_flags(res);
    cpu.r.y = res;
}

fn top(cpu: &mut Cpu, addressing_mode: usize) {
    let (_addr, page_crossing) = addressing_modes::FUNCTION_TABLE[addressing_mode](cpu);
    if page_crossing {
        cpu.cycle += 1;
    }
}

fn tsx(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.sp;
    cpu.r.update_nz_flags(res);
    cpu.r.x = res;
}

fn txa(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.x;
    cpu.r.update_nz_flags(res);
    cpu.r.a = res;
}

fn txs(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.x;
    cpu.r.sp = res;
}

fn tya(cpu: &mut Cpu, _addressing_mode: usize) {
    let res = cpu.r.y;
    cpu.r.update_nz_flags(res);
    cpu.r.a = res;
}

fn xaa(cpu: &mut Cpu, addressing_mode: usize) {
    let operand = cpu.get_operand(addressing_mode);
    let res = cpu.r.x & operand.val;
    cpu.r.a = res;
}
