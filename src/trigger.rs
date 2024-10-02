use std::marker::PhantomData;

use libfabric_sys::{fi_trigger_var__bindgen_ty_1, fi_trigger_xpu__bindgen_ty_1};

use crate::{
    cntr::Counter,
    enums::{HmemIface, TriggerEvent},
    fid::{AsRawTypedFid, CntrRawFid},
};

pub(crate) struct TriggeredContext1<'a, 'b> {
    #[allow(dead_code)]
    c_val: libfabric_sys::fi_triggered_context,
    phantom: PhantomData<&'a mut TriggerEvent<'b>>,
}

impl<'a, 'b> TriggeredContext1<'a, 'b> {
    pub fn new(event: &'a mut TriggerEvent<'b>) -> Self {
        let (event_type, trigger) = event.as_raw();
        Self {
            c_val: {
                libfabric_sys::fi_triggered_context {
                    event_type,
                    trigger,
                }
            },
            phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_triggered_context {
        &self.c_val
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_triggered_context {
        &mut self.c_val
    }
}

// impl Default for TriggeredContext1 {
//     fn default() -> Self {
//         Self::new()
//     }
// }

pub(crate) struct TriggeredContext2<'a, 'b> {
    #[allow(dead_code)]
    c_val: libfabric_sys::fi_triggered_context2,
    phantom: PhantomData<&'a mut TriggerEvent<'b>>,
}

impl<'a, 'b> TriggeredContext2<'a, 'b> {
    pub fn new(event: &'a mut TriggerEvent<'b>) -> Self {
        let (event_type, trigger) = event.as_raw2();
        Self {
            c_val: {
                libfabric_sys::fi_triggered_context2 {
                    event_type,
                    trigger,
                }
            },
            phantom: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_triggered_context2 {
        &mut self.c_val
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_triggered_context2 {
        &self.c_val
    }
}

pub(crate) enum TriggeredContextType<'a, 'b> {
    TriggeredContext1(Box<TriggeredContext1<'a, 'b>>),
    TriggeredContext2(Box<TriggeredContext2<'a, 'b>>),
}

// We use heap allocated data to allow moving the wrapper field
// without affecting the pointer used by libfabric

pub struct TriggeredContext<'a, 'b>(pub(crate) TriggeredContextType<'a, 'b>);

impl<'a, 'b> TriggeredContext<'a, 'b> {
    pub(crate) fn inner_mut(&mut self) -> *mut std::ffi::c_void {
        match &mut self.0 {
            TriggeredContextType::TriggeredContext1(ctx) => ctx.get_mut() as *mut std::ffi::c_void,
            TriggeredContextType::TriggeredContext2(ctx) => ctx.get_mut() as *mut std::ffi::c_void,
        }
    }

    pub(crate) fn inner(&self) -> *const std::ffi::c_void {
        match &self.0 {
            TriggeredContextType::TriggeredContext1(ctx) => ctx.get() as *const std::ffi::c_void,
            TriggeredContextType::TriggeredContext2(ctx) => ctx.get() as *const std::ffi::c_void,
        }
    }
}

pub struct TriggerThreshold<'a> {
    pub(crate) c_thold: libfabric_sys::fi_trigger_threshold,
    phantom: PhantomData<&'a ()>,
}

impl<'a> TriggerThreshold<'a> {
    pub fn new<T: AsRawTypedFid<Output = CntrRawFid>>(
        cntr: &'a Counter<T>,
        threshold: usize,
    ) -> Self {
        Self {
            c_thold: libfabric_sys::fi_trigger_threshold {
                cntr: cntr.as_raw_typed_fid(),
                threshold,
            },
            phantom: PhantomData,
        }
    }
}
pub enum TriggerVal {
    U8(Vec<u8>),
    U16(Vec<u16>),
    U32(Vec<u32>),
    U64(Vec<u64>),
}
// #[repr(C)]
// pub struct TriggerVar {
//     c_trigger_var: libfabric_sys::fi_trigger_var,
// }

// pub trait TriggerVal {
//     fn fi_datatype(&self) -> (libfabric_sys::fi_datatype, libfabric_sys::fi_trigger_var__bindgen_ty_1);
// }

// impl TriggerVal for u8{
//     fn fi_datatype(&self) -> (libfabric_sys::fi_datatype, libfabric_sys::fi_trigger_var__bindgen_ty_1) {
//         (libfabric_sys::fi_datatype_FI_UINT8, libfabric_sys::fi_trigger_var__bindgen_ty_1{val8: *self})
//     }
// }

// impl TriggerVal for u16{
//     fn fi_datatype(&self) -> (libfabric_sys::fi_datatype, libfabric_sys::fi_trigger_var__bindgen_ty_1) {
//         (libfabric_sys::fi_datatype_FI_UINT16, libfabric_sys::fi_trigger_var__bindgen_ty_1{val16: *self})
//     }
// }

// impl TriggerVal for u32{
//     fn fi_datatype(&self) -> (libfabric_sys::fi_datatype, libfabric_sys::fi_trigger_var__bindgen_ty_1) {
//         (libfabric_sys::fi_datatype_FI_UINT32, libfabric_sys::fi_trigger_var__bindgen_ty_1{val32: *self})
//     }
// }

// impl TriggerVal for u64{
//     fn fi_datatype(&self) -> (libfabric_sys::fi_datatype, libfabric_sys::fi_trigger_var__bindgen_ty_1) {
//         (libfabric_sys::fi_datatype_FI_UINT64, libfabric_sys::fi_trigger_var__bindgen_ty_1{val64: *self})
//     }
// }

// impl TriggerVar {
//     pub fn new<T: TriggerVal>(val: &T) -> Self {
//         let (datatype, value) = val.fi_datatype();
//         Self {
//             c_trigger_var: libfabric_sys::fi_trigger_var {
//                 datatype,
//                 count: 1,
//                 addr: std::ptr::null_mut(),
//                 value,
//             }
//         }
//     }
// }

pub struct TriggerXpu {
    hmem_iface: HmemIface,
    trigger_vars: Vec<libfabric_sys::fi_trigger_var>,
    trigger_vars_data: Vec<Vec<u8>>,
}

impl TriggerXpu {
    pub fn new(iface: HmemIface, trigger_vars: Vec<libfabric_sys::fi_trigger_var>) -> Self {
        let trigger_vars_data: Vec<Vec<u8>> = trigger_vars
            .iter()
            .map(|v| {
                // if v.count > 1 {
                if v.datatype == libfabric_sys::fi_datatype_FI_UINT8 {
                    vec![0u8; v.count as usize]
                } else if v.datatype == libfabric_sys::fi_datatype_FI_UINT16 {
                    vec![0u8; 2 * v.count as usize]
                } else if v.datatype == libfabric_sys::fi_datatype_FI_UINT32 {
                    vec![0u8; 32 * v.count as usize]
                } else if v.datatype == libfabric_sys::fi_datatype_FI_UINT64 {
                    vec![0u8; 8 * v.count as usize]
                } else {
                    panic!("Unexpected datatype")
                }
                // } else {
                //     None
                // }
            })
            .collect();

        let mut res = Self {
            trigger_vars,
            hmem_iface: iface,
            trigger_vars_data,
        };

        let mut i = 0;
        for var in res.trigger_vars.iter_mut() {
            if var.count > 1 {
                var.value = fi_trigger_var__bindgen_ty_1 {
                    data: res.trigger_vars_data[i].as_mut_ptr(),
                };
                i += 1;
            }
        }

        res
    }

    pub fn vals_and_addresses(&mut self) -> Vec<(TriggerVal, usize)> {
        self.trigger_vars
            .iter()
            .filter_map(|v| {
                if v.addr as usize == 0 {
                    None
                } else {
                    Some((
                        if v.datatype == libfabric_sys::fi_datatype_FI_UINT8 {
                            TriggerVal::U8(if v.count == 1 {
                                vec![unsafe { v.value.val8 }]
                            } else {
                                unsafe {
                                    std::slice::from_raw_parts(v.value.data, v.count as usize)
                                        .to_vec()
                                }
                            })
                        } else if v.datatype == libfabric_sys::fi_datatype_FI_UINT16 {
                            TriggerVal::U16(if v.count == 1 {
                                vec![unsafe { v.value.val16 }]
                            } else {
                                unsafe {
                                    std::slice::from_raw_parts(
                                        v.value.data as *mut u16,
                                        v.count as usize,
                                    )
                                    .to_vec()
                                }
                            })
                        } else if v.datatype == libfabric_sys::fi_datatype_FI_UINT32 {
                            TriggerVal::U32(if v.count == 1 {
                                vec![unsafe { v.value.val32 }]
                            } else {
                                unsafe {
                                    std::slice::from_raw_parts(
                                        v.value.data as *mut u32,
                                        v.count as usize,
                                    )
                                    .to_vec()
                                }
                            })
                        } else if v.datatype == libfabric_sys::fi_datatype_FI_UINT64 {
                            TriggerVal::U64(if v.count == 1 {
                                vec![unsafe { v.value.val64 }]
                            } else {
                                unsafe {
                                    std::slice::from_raw_parts(
                                        v.value.data as *mut u64,
                                        v.count as usize,
                                    )
                                    .to_vec()
                                }
                            })
                        } else {
                            panic!("Unexpected datatype")
                        },
                        v.addr as usize,
                    ))
                }
            })
            .collect()
    }

    pub(crate) fn as_raw(&mut self) -> libfabric_sys::fi_trigger_xpu {
        let (dev_id, dev_type) = match self.hmem_iface {
            HmemIface::Cuda(id) => (
                fi_trigger_xpu__bindgen_ty_1 { cuda: id },
                libfabric_sys::fi_hmem_iface_FI_HMEM_CUDA,
            ),
            HmemIface::Ze(drv_id, dev_id) => (
                fi_trigger_xpu__bindgen_ty_1 {
                    ze: unsafe { libfabric_sys::inlined_fi_hmem_ze_device(drv_id, dev_id) },
                },
                libfabric_sys::fi_hmem_iface_FI_HMEM_ZE,
            ),
            _ => panic!("Device type not supported"),
        };

        libfabric_sys::fi_trigger_xpu {
            count: self.trigger_vars.len() as i32,
            iface: dev_type,
            device: dev_id,
            var: self.trigger_vars.as_mut_ptr().cast(),
        }
    }
}
