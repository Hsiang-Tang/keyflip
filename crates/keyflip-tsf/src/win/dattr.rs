//! Display attributes: how the application draws our composition text.

use std::cell::Cell;

use windows::core::{implement, Result, BSTR, GUID};
use windows::Win32::Foundation::{BOOL, E_INVALIDARG, S_FALSE};
use windows::Win32::UI::TextServices::{
    IEnumTfDisplayAttributeInfo, IEnumTfDisplayAttributeInfo_Impl, ITfDisplayAttributeInfo,
    ITfDisplayAttributeInfo_Impl, TF_ATTR_INPUT, TF_ATTR_TARGET_CONVERTED, TF_CT_NONE,
    TF_CT_SYSCOLOR, TF_DA_COLOR, TF_DA_COLOR_0, TF_DISPLAYATTRIBUTE, TF_LS_NONE, TF_LS_SOLID,
};

use super::Live;
use crate::consts::{DA_INPUT_U128, DA_SELECTED_U128};

pub fn input_guid() -> GUID {
    GUID::from_u128(DA_INPUT_U128)
}

pub fn selected_guid() -> GUID {
    GUID::from_u128(DA_SELECTED_U128)
}

const COLOR_HIGHLIGHT: i32 = 13;
const COLOR_HIGHLIGHTTEXT: i32 = 14;

fn none_color() -> TF_DA_COLOR {
    TF_DA_COLOR {
        r#type: TF_CT_NONE,
        Anonymous: TF_DA_COLOR_0 { nIndex: 0 },
    }
}

fn sys_color(index: i32) -> TF_DA_COLOR {
    TF_DA_COLOR {
        r#type: TF_CT_SYSCOLOR,
        Anonymous: TF_DA_COLOR_0 { nIndex: index },
    }
}

/// Underline, no colors.
fn input_attr() -> TF_DISPLAYATTRIBUTE {
    TF_DISPLAYATTRIBUTE {
        crText: none_color(),
        crBk: none_color(),
        lsStyle: TF_LS_SOLID,
        fBoldLine: BOOL(0),
        crLine: none_color(),
        bAttr: TF_ATTR_INPUT,
    }
}

/// Highlighted, no underline.
fn selected_attr() -> TF_DISPLAYATTRIBUTE {
    TF_DISPLAYATTRIBUTE {
        crText: sys_color(COLOR_HIGHLIGHTTEXT),
        crBk: sys_color(COLOR_HIGHLIGHT),
        lsStyle: TF_LS_NONE,
        fBoldLine: BOOL(0),
        crLine: none_color(),
        bAttr: TF_ATTR_TARGET_CONVERTED,
    }
}

#[implement(ITfDisplayAttributeInfo)]
pub struct DaInfo {
    guid: GUID,
    desc: &'static str,
    attr: TF_DISPLAYATTRIBUTE,
    _live: Live,
}

impl DaInfo {
    pub fn for_guid(guid: &GUID) -> Option<ITfDisplayAttributeInfo> {
        if *guid == input_guid() {
            Some(
                DaInfo {
                    guid: input_guid(),
                    desc: "keyflip input",
                    attr: input_attr(),
                    _live: Live::new(),
                }
                .into(),
            )
        } else if *guid == selected_guid() {
            Some(
                DaInfo {
                    guid: selected_guid(),
                    desc: "keyflip selected",
                    attr: selected_attr(),
                    _live: Live::new(),
                }
                .into(),
            )
        } else {
            None
        }
    }
}

impl ITfDisplayAttributeInfo_Impl for DaInfo_Impl {
    fn GetGUID(&self) -> Result<GUID> {
        Ok(self.guid)
    }

    fn GetDescription(&self) -> Result<BSTR> {
        Ok(BSTR::from(self.desc))
    }

    fn GetAttributeInfo(&self, pda: *mut TF_DISPLAYATTRIBUTE) -> Result<()> {
        if pda.is_null() {
            return Err(E_INVALIDARG.into());
        }
        unsafe { *pda = self.attr };
        Ok(())
    }

    // Attributes are fixed; applications may try to customise them.
    fn SetAttributeInfo(&self, _pda: *const TF_DISPLAYATTRIBUTE) -> Result<()> {
        Ok(())
    }

    fn Reset(&self) -> Result<()> {
        Ok(())
    }
}

#[implement(IEnumTfDisplayAttributeInfo)]
pub struct DaEnum {
    pos: Cell<usize>,
    _live: Live,
}

impl DaEnum {
    pub fn new() -> IEnumTfDisplayAttributeInfo {
        DaEnum { pos: Cell::new(0), _live: Live::new() }.into()
    }
}

impl IEnumTfDisplayAttributeInfo_Impl for DaEnum_Impl {
    fn Clone(&self) -> Result<IEnumTfDisplayAttributeInfo> {
        Ok(DaEnum {
            pos: Cell::new(self.pos.get()),
            _live: Live::new(),
        }
        .into())
    }

    fn Next(
        &self,
        ulcount: u32,
        rginfo: *mut Option<ITfDisplayAttributeInfo>,
        pcfetched: *mut u32,
    ) -> Result<()> {
        let guids = [input_guid(), selected_guid()];
        let mut n = 0u32;
        while n < ulcount && self.pos.get() < guids.len() {
            let info = DaInfo::for_guid(&guids[self.pos.get()]);
            unsafe { rginfo.add(n as usize).write(info) };
            self.pos.set(self.pos.get() + 1);
            n += 1;
        }
        if !pcfetched.is_null() {
            unsafe { *pcfetched = n };
        }
        if n == ulcount {
            Ok(())
        } else {
            Err(S_FALSE.into())
        }
    }

    fn Reset(&self) -> Result<()> {
        self.pos.set(0);
        Ok(())
    }

    fn Skip(&self, ulcount: u32) -> Result<()> {
        self.pos.set((self.pos.get() + ulcount as usize).min(2));
        Ok(())
    }
}
