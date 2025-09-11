use crate::enums::AtomicOp;
use crate::async_::cq::AsyncReadCq;
use crate::async_::eq::AsyncReadEq;
use crate::async_::xcontext::{TxContext, TxContextImpl};
// use crate::async_::xcontext::{TxContext, TxContextImpl};
use crate::comm::atomic::{AtomicCASImpl, AtomicFetchEpImpl};
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::enums::{AtomicFetchMsgOptions, AtomicMsgOptions};
use crate::ep::{Connected, Connectionless, EndpointBase, EndpointImplBase, EpState};
use crate::infocapsoptions::{AtomicCap, ReadMod, WriteMod};
use crate::mr::{MemoryRegionDesc, MemoryRegionSlice, MemoryRegionSliceMut};
use crate::utils::Either;
use crate::{
    async_::ep::AsyncTxEp, comm::atomic::AtomicWriteEpImpl, cq::SingleCompletion,
    mr::MappedMemoryRegionKey, AsFiType, Context,
};
use crate::{AsFiOrBoolType, RemoteMemAddrSlice, RemoteMemAddrSliceMut, RemoteMemoryAddress};

use super::while_try_again;

pub(crate) trait AsyncAtomicWriteEpImpl: AtomicWriteEpImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async fn atomic_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
        op: crate::enums::AtomicOp,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.atomic_impl(
                buf,
                desc,
                dest_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
                op,
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn inject_atomic_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        buf: &[T],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        op: crate::enums::AtomicOp,
    ) -> Result<(), crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.inject_atomic_impl(buf, dest_addr, mem_addr, mapped_key, op)
        })
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn atomicv_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
        op: crate::enums::AtomicOp,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.atomicv_impl(
                ioc,
                desc,
                dest_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
                op,
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgAtomic<'_, T>,
            &mut crate::msg::MsgAtomicConnected<'_, T>,
        >,
        options: AtomicMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.atomicmsg_impl(imm_msg.to_owned(), options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}


macro_rules! gen_atomic_op_decl_single {
    ($res_type:ty, $func:ident (< $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >),  ($self: ident, $($p: ident : $t: ty),*)) =>
    {
        unsafe fn $func< $( $N $(: $b0 $(+$b)* )? ),* >
        (
            &$self,
            $($p: $t),*
        ) ->  impl std::future::Future<Output = Result<$res_type, crate::error::Error>>;
    };

    ($res_type:ty, $func:ident (),  $($p_and_t: tt)*) => {
        gen_atomic_op_decl_single!($res_type, $func (<>), $($p_and_t),*);
    };
}

macro_rules! gen_atomic_op_decl {
    ($gen: tt, $args: tt -> $res_type:ty, $($func:ident),+) =>
    {
        $(
            gen_atomic_op_decl_single!($res_type, $func $gen, $args);
        )+
    }
}

macro_rules! gen_atomic_op_def_single {
    ($res_type:ty, $func:ident (< $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >),  ($self: ident, $($p: ident : $t: ty),*), $inner_func:ident ($($vals: expr),*), $op: path) =>
    {
        #[inline]
        unsafe fn $func< $( $N $(: $b0 $(+$b)* )? ),* >
        (
            &$self,
            $($p: $t),*
        ) -> impl std::future::Future<Output = Result<$res_type, crate::error::Error>>
        {
            $self.$inner_func($($vals,)* $op)
        }
    };

    ($res_type:ty, $func:ident (),  $p_and_t: tt, $inner_func:ident $vals: tt, $op: path) => {
        gen_atomic_op_def_single!($res_type, $func (<>), $p_and_t, $inner_func $vals, $op);
    };
}

macro_rules! gen_atomic_op_def {
    ($gen: tt, $args: tt -> $res_type:ty, $inner_func: ident $vals: tt, $($op: path,)+, $($func:ident),+) =>
    {
        $(
            gen_atomic_op_def_single!($res_type, $func $gen, $args, $inner_func $vals, $op);
        )+
    }
}



pub trait AsyncAtomicWriteEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomic_min_to_async, atomic_max_to_async, atomic_sum_to_async,  atomic_prod_to_async, atomic_bor_to_async, atomic_band_to_async, atomic_bxor_to_async
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomic_lor_to_async, atomic_land_to_async, atomic_lxor_to_async
    );

    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered 
    // );

    // gen_atomic_op_decl!((), (
    //     self,
    //     buf: &[bool],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered
    // );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomicv_min_to_async, atomicv_max_to_async, atomicv_sum_to_async,  atomicv_prod_to_async, atomicv_bor_to_async, atomicv_band_to_async, atomicv_bxor_to_async
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomicv_lor_to_async, atomicv_land_to_async, atomicv_lxor_to_async
    );

    unsafe fn atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;


    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    )-> (),
    atomic_inject_min_to_async, atomic_inject_max_to_async, atomic_inject_sum_to_async,  atomic_inject_prod_to_async, atomic_inject_bor_to_async, atomic_inject_band_to_async, atomic_inject_bxor_to_async
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    )-> (),
    atomic_inject_lor_to_async, atomic_inject_land_to_async, atomic_inject_lxor_to_async
    );
}

pub trait ConnectedAsyncAtomicWriteEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomic_min_async, atomic_max_async, atomic_sum_async, atomic_prod_async, atomic_bor_async, atomic_band_async, atomic_bxor_async
    );
    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomic_lor_async, atomic_land_async, atomic_lxor_async
    );
    
    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ), 
    // atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered, atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered
    // );

    // gen_atomic_op_decl!((), (
    //     self,
    //     buf: &[bool],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ), 
    // atomic_lor_triggered, atomic_land_triggered, atomic_lxor_triggered
    // );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomicv_min_async, atomicv_max_async, atomicv_sum_async, atomicv_prod_async, atomicv_bor_async, atomicv_band_async, atomicv_bxor_async
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    atomicv_lor_async, atomicv_land_async, atomicv_lxor_async
    );

    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // atomicv_min_triggered, atomicv_max_triggered, atomicv_sum_triggered, atomicv_prod_triggered, atomicv_bor_triggered, atomicv_band_triggered, atomicv_bxor_triggered
    // );

    // gen_atomic_op_decl!((), (
    //     self,
    //     ioc: &[crate::iovec::Ioc<bool>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // atomicv_lor_triggered, atomicv_land_triggered, atomicv_lxor_triggered
    // );
    unsafe fn atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;


    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    )-> (),
    atomic_inject_min_async, atomic_inject_max_async, atomic_inject_sum_async, atomic_inject_prod_async, atomic_inject_bor_async, atomic_inject_band_async, atomic_inject_bxor_async
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    )-> (),
    atomic_inject_lor_async, atomic_inject_land_async, atomic_inject_lxor_async
    );
}


macro_rules! gen_atomic_mr_op_def_single {
    ($res_type: ty, $func:ident (< $( $N:ident $(: $b0:ident $(+$b:ident)* )? ),* >),  ($self: ident, $($p: ident : $t: ty),*), ($($vals: expr),*), $base_func: ident) =>
    {
        unsafe fn $func< $( $N $(: $b0 $(+$b)* )? ),* >
        (
            &$self,
            $($p: $t),*
        ) -> impl std::future::Future<Output = Result<$res_type, crate::error::Error>>
        {
            $self.$base_func($($vals,)*)
        }
    };

    ($res_type: ty, $func:ident (),  $p_and_t: tt, $vals: tt, $base_func: ident) => {
        gen_atomic_mr_op_def_single!($res_type, $func (<>), $p_and_t, $vals, $base_func);
    };
}

macro_rules! gen_atomic_mr_op_def {
    ($gen: tt, $args: tt -> $res_type: ty, $vals: tt, $($base_func: ident,)+, $($func:ident),+) =>
    {
        $(
            gen_atomic_mr_op_def_single!($res_type, $func $gen, $args, $vals, $base_func);
        )+
    }
}


pub trait AsyncAtomicWriteEpMrSlice : AsyncAtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
        (mr_slice.as_slice(), Some(mr_slice.desc()), dest_addr, mem_addr, mapped_key, context),
        atomic_min_to_async, atomic_max_to_async, atomic_sum_to_async,  atomic_prod_to_async, atomic_bor_to_async, atomic_band_to_async, atomic_bxor_to_async,, atomic_mr_min_to_async, atomic_mr_max_to_async, atomic_mr_sum_to_async,  atomic_mr_prod_to_async, atomic_mr_bor_to_async, atomic_mr_band_to_async, atomic_mr_bxor_to_async
    );

    // gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
    //     self,
    //     mr_slice: &MemoryRegionSlice,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion ,
    //     (mr_slice.as_slice(), Some(mr_slice.desc()), dest_addr, mem_addr, mapped_key, context),
    //     atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered,, atomic_mr_min_to_triggered, atomic_mr_max_to_triggered, atomic_mr_sum_to_triggered,  atomic_mr_prod_to_triggered, atomic_mr_bor_to_triggered, atomic_mr_band_to_triggered, atomic_mr_bxor_to_triggered
    // );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ) -> (),
        (mr_slice.as_slice(), dest_addr, mem_addr, mapped_key),
        atomic_inject_min_to_async, atomic_inject_max_to_async, atomic_inject_sum_to_async,  atomic_inject_prod_to_async, atomic_inject_bor_to_async, atomic_inject_band_to_async, atomic_inject_bxor_to_async,, atomic_mr_inject_min_to_async, atomic_mr_inject_max_to_async, atomic_mr_inject_sum_to_async,  atomic_mr_inject_prod_to_async, atomic_mr_inject_bor_to_async, atomic_mr_inject_band_to_async, atomic_mr_inject_bxor_to_async
    );
    // #[allow(clippy::too_many_arguments)]
    // unsafe fn atomic_mr_slice_to_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     mr_slice: &MemoryRegionSlice,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     self.atomic_to_async(mr_slice.as_slice(), Some(mr_slice.desc()), dest_addr, mem_addr, mapped_key, op, context)
    // }

    // #[allow(clippy::too_many_arguments)]
    // unsafe fn inject_atomic_mr_slice_to_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     mr_slice: &MemoryRegionSlice,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    // ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
    //     self.inject_atomic_to_async(mr_slice.as_slice(), dest_addr, mem_addr, mapped_key, op)
    // }
}

pub trait ConnectedAsyncAtomicWriteEpMrSlice: ConnectedAsyncAtomicWriteEp {
    // #[allow(clippy::too_many_arguments)]
    // unsafe fn atomic_mr_slice_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     mr_slice: &MemoryRegionSlice,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     self.atomic_async(
    //         mr_slice.as_slice() ,
    //         Some(mr_slice.desc()),
    //         mem_addr,
    //         mapped_key,
    //         op,
    //         context,
    //     )
    // }

    // #[allow(clippy::too_many_arguments)]
    // unsafe fn inject_atomic_mr_slice_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     mr_slice: &MemoryRegionSlice,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    // ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
    //     self.inject_atomic_async(
    //         mr_slice.as_slice() ,
    //         mem_addr,
    //         mapped_key,
    //         op,
    //     )
    // }

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
        (mr_slice.as_slice(), Some(mr_slice.desc()), mem_addr, mapped_key, context),
        atomic_min_async, atomic_max_async, atomic_sum_async,  atomic_prod_async, atomic_bor_async, atomic_band_async, atomic_bxor_async,, atomic_mr_min_async, atomic_mr_max_async, atomic_mr_sum_async,  atomic_mr_prod_async, atomic_mr_bor_async, atomic_mr_band_async, atomic_mr_bxor_async
    );


    // gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
    //     self,
    //     mr_slice: &MemoryRegionSlice,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    //     (mr_slice.as_slice(), Some(mr_slice.desc()), mem_addr, mapped_key, context),
    //     atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered,  atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered,, atomic_mr_min_triggered, atomic_mr_max_triggered, atomic_mr_sum_triggered,  atomic_mr_prod_triggered, atomic_mr_bor_triggered, atomic_mr_band_triggered, atomic_mr_bxor_triggered
    // );

    gen_atomic_mr_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        mr_slice: &MemoryRegionSlice,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    ) -> (),
        (mr_slice.as_slice(), mem_addr, mapped_key),
        atomic_inject_min_async, atomic_inject_max_async, atomic_inject_sum_async,  atomic_inject_prod_async, atomic_inject_bor_async, atomic_inject_band_async, atomic_inject_bxor_async,, atomic_mr_inject_min, atomic_mr_inject_max, atomic_mr_inject_sum,  atomic_mr_inject_prod, atomic_mr_inject_bor, atomic_mr_inject_band, atomic_mr_inject_bxor
    );
}

impl<EP: ConnectedAsyncAtomicWriteEp> ConnectedAsyncAtomicWriteEpMrSlice for EP {}

impl<E: AsyncAtomicWriteEpImpl> AsyncAtomicWriteEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicWriteEpImpl> AsyncAtomicWriteEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + WriteMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicWriteEpImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + WriteMod, STATE: EpState> AsyncAtomicWriteEpImpl for TxContextImpl<I, STATE> {}

impl<I: AtomicCap + WriteMod, STATE: EpState> AsyncAtomicWriteEpImpl for TxContext<I, STATE> {}

impl<EP: AsyncAtomicWriteEpImpl + ConnlessEp> AsyncAtomicWriteEp for EP {
    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
        atomic_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, context), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min_to_async, atomic_max_to_async, atomic_sum_to_async, atomic_prod_to_async, atomic_bor_to_async, atomic_band_to_async, atomic_bxor_to_async
    );
    
    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
        atomic_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, context), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor_to_async, atomic_land_to_async, atomic_lxor_to_async
    );


    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    //     atomic_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, context),  AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
    //     atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered, atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered
    // );

    // gen_atomic_op_def!((), ( 
    //     self,
    //     buf: &[bool],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    //     atomic_async_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, context),  AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
    //     atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered
    // );


    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context
    )-> SingleCompletion,
        atomicv_async_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, ctx), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomicv_min_to_async, atomicv_max_to_async, atomicv_sum_to_async, atomicv_prod_to_async, atomicv_bor_to_async, atomicv_band_to_async, atomicv_bxor_to_async
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context
    )-> SingleCompletion,
        atomicv_async_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, ctx), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomicv_lor_to_async, atomicv_land_to_async, atomicv_lxor_to_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     ctx: &mut TriggeredContext
    // ),
    //     atomicv_async_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, ctx), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
    //     atomicv_min_to_triggered, atomicv_max_to_triggered, atomicv_sum_to_triggered, atomicv_prod_to_triggered, atomicv_bor_to_triggered, atomicv_band_to_triggered, atomicv_bxor_to_triggered
    // );

    // gen_atomic_op_def!((), ( 
    //     self,
    //     ioc: &[crate::iovec::Ioc<bool>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     ctx: &mut TriggeredContext
    // ),
    //     atomicv_async_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, ctx), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
    //     atomicv_lor_to_triggered, atomicv_land_to_triggered, atomicv_lxor_to_triggered
    // );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    )-> (),
        inject_atomic_async_impl(buf, Some(dest_addr), mem_addr, mapped_key), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,,
        atomic_inject_min_to_async, atomic_inject_max_to_async, atomic_inject_sum_to_async, atomic_inject_prod_to_async, atomic_inject_bor_to_async, atomic_inject_band_to_async, atomic_inject_bxor_to_async
    );

    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    )-> (),
        inject_atomic_async_impl(buf, Some(dest_addr), mem_addr, mapped_key), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,,
        atomic_inject_lor_to_async, atomic_inject_land_to_async, atomic_inject_lxor_to_async
    );
    // #[inline]
    // unsafe fn atomic_to_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     self.atomic_async_impl(
    //         buf,
    //         desc,
    //         Some(dest_addr),
    //         mem_addr,
    //         mapped_key,
    //         op,
    //         context,
    //     )
    // }

    // #[inline]
    // unsafe fn inject_atomic_to_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     buf: &[T],
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    // ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
    //     self.inject_atomic_async_impl(buf, Some(dest_addr), mem_addr, mapped_key, op)
    // }

    // #[inline]
    // unsafe fn atomicv_to_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     self.atomicv_async_impl(
    //         ioc,
    //         desc,
    //         Some(dest_addr),
    //         mem_addr,
    //         mapped_key,
    //         op,
    //         context,
    //     )
    // }

    #[inline]
    unsafe fn atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async_impl(Either::Left(msg), options)
    }
}

impl<EP: AsyncAtomicWriteEpImpl + ConnectedEp> ConnectedAsyncAtomicWriteEp for EP {
    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        atomic_async_impl(buf, desc, None, mem_addr, mapped_key, context), AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_min_async, atomic_max_async, atomic_sum_async, atomic_prod_async, atomic_bor_async, atomic_band_async, atomic_bxor_async
    );
    
    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        atomic_async_impl(buf, desc, None, mem_addr, mapped_key, context), AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_lor_async, atomic_land_async, atomic_lxor_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // )->SingleCompletion,
    //     atomic_impl(
    //         buf,
    //         desc,
    //         None,
    //         mem_addr,
    //         mapped_key,
    //         Some(context.inner_mut())
    //     ),
    //     AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
    //     atomic_min_triggered, atomic_max_triggered, atomic_sum_triggered, atomic_prod_triggered, atomic_bor_triggered, atomic_band_triggered, atomic_bxor_triggered
    // );

    // gen_atomic_op_def!((), ( 
    //     self,
    //     buf: &[bool],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // )->SingleCompletion,
    //     atomic_impl(
    //         buf,
    //         desc,
    //         None,
    //         mem_addr,
    //         mapped_key,
    //         Some(context.inner_mut())
    //     ),
    //     AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
    //     atomic_lor_triggered, atomic_land_triggered, atomic_lxor_triggered
    // );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion, 
        atomicv_async_impl(ioc, desc, None, mem_addr, mapped_key, context),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomicv_min_async, atomicv_max_async, atomicv_sum_async, atomicv_prod_async, atomicv_bor_async, atomicv_band_async, atomicv_bxor_async
    );

    gen_atomic_op_def!((), ( 
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        atomicv_async_impl(ioc, desc, None, mem_addr, mapped_key, context),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomicv_lor_async, atomicv_land_async, atomicv_lxor_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // )->SingleCompletion,
    //     atomicv_impl(ioc, desc, None, mem_addr, mapped_key, Some(context.inner_mut())),
    //     AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
    //     atomicv_min_triggered, atomicv_max_triggered, atomicv_sum_triggered, atomicv_prod_triggered, atomicv_bor_triggered, atomicv_band_triggered, atomicv_bxor_triggered
    // );

    // gen_atomic_op_def!((), ( 
    //     self,
    //     ioc: &[crate::iovec::Ioc<bool>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     mem_addr: RemoteMemoryAddress<bool>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // )->SingleCompletion,
    //     atomicv_impl(ioc, desc, None, mem_addr, mapped_key, Some(context.inner_mut())),
    //     AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
    //     atomicv_lor_triggered, atomicv_land_triggered, atomicv_lxor_triggered
    // );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), ( 
        self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey
    )->(),
        inject_atomic_async_impl(buf, None, mem_addr, mapped_key),
        AtomicOp::Min, AtomicOp::Max, AtomicOp::Sum, AtomicOp::Prod, AtomicOp::Bor, AtomicOp::Band, AtomicOp::Bxor,, 
        atomic_inject_min_async, atomic_inject_max_async, atomic_inject_sum_async, atomic_inject_prod_async, atomic_inject_bor_async, atomic_inject_band_async, atomic_inject_bxor_async
    );

    gen_atomic_op_def!((), ( 
        self,
        buf: &[bool],
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey
    )->(),
        inject_atomic_async_impl(buf, None, mem_addr, mapped_key),
        AtomicOp::Lor, AtomicOp::Land, AtomicOp::Lxor,, 
        atomic_inject_lor_async, atomic_inject_land_async, atomic_inject_lxor_async
    );

    unsafe fn atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async_impl(Either::Right(msg), options)
    }
}

pub trait AsyncAtomicWriteRemoteMemAddrSliceEp: AsyncAtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion,   
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_min_to_async, atomic_max_to_async, atomic_sum_to_async,  atomic_prod_to_async, atomic_bor_to_async, atomic_band_to_async, atomic_bxor_to_async,, atomic_min_mr_slice_to_async, atomic_max_mr_slice_to_async, atomic_sum_mr_slice_to_async,  atomic_prod_mr_slice_to_async, atomic_bor_mr_slice_to_async, atomic_band_mr_slice_to_async, atomic_bxor_mr_slice_to_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_lor_to_async, atomic_land_to_async, atomic_lxor_to_async,, atomic_lor_mr_slice_to_async, atomic_land_mr_slice_to_async, atomic_lxor_mr_slice_to_async
    );

    // gen_atomic_mr_op_def!((<T: AsFiType>), (
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     dest_slice: &RemoteMemAddrSliceMut<T>,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion,
    //     (
    //         buf,
    //         desc,
    //         dest_addr,
    //         dest_slice.mem_address(),
    //         dest_slice.key(),
    //         context
    //     ),
    //     atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered,, atomic_mr_slice_min_to_triggered, atomic_mr_slice_max_to_triggered, atomic_mr_slice_sum_to_triggered,  atomic_mr_slice_prod_to_triggered, atomic_mr_slice_bor_to_triggered, atomic_mr_slice_band_to_triggered, atomic_mr_slice_bxor_to_triggered
    // );

    // gen_atomic_mr_op_def!((), (
    //     self,
    //     buf: &[bool],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     dest_slice: &RemoteMemAddrSliceMut<bool>,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion,   
    //     (
    //         buf,
    //         desc,
    //         dest_addr,
    //         dest_slice.mem_address(),
    //         dest_slice.key(),
    //         context
    //     ),
    //     atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered,, atomic_mr_slice_lor_to_triggered, atomic_mr_slice_land_to_triggered, atomic_mr_slice_lxor_to_triggered
    // );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion, 
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_min_to_async, atomicv_max_to_async, atomicv_sum_to_async,  atomicv_prod_to_async, atomicv_bor_to_async, atomicv_band_to_async, atomicv_bxor_to_async,, atomicv_min_mr_slice_to_async, atomicv_max_mr_slice_to_async, atomicv_sum_mr_slice_to_async,  atomicv_prod_mr_slice_to_async, atomicv_bor_mr_slice_to_async, atomicv_band_mr_slice_to_async, atomicv_bxor_mr_slice_to_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_lor_to_async, atomicv_land_to_async, atomicv_lxor_to_async,, atomicv_lor_mr_slice_to_async, atomicv_land_mr_slice_to_async, atomicv_lxor_mr_slice_to_async
    );

    // gen_atomic_mr_op_def!((<T: AsFiType>), (
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     dest_slice: &RemoteMemAddrSliceMut<T>,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion,
    //     (
    //         ioc,
    //         desc,
    //         dest_addr,
    //         dest_slice.mem_address(),
    //         dest_slice.key(),
    //         context
    //     ),
    //     atomicv_min_to_triggered, atomicv_max_to_triggered, atomicv_sum_to_triggered,  atomicv_prod_to_triggered, atomicv_bor_to_triggered, atomicv_band_to_triggered, atomicv_bxor_to_triggered,, atomicv_mr_slice_min_to_triggered, atomicv_mr_slice_max_to_triggered, atomicv_mr_slice_sum_to_triggered,  atomicv_mr_slice_prod_to_triggered, atomicv_mr_slice_bor_to_triggered, atomicv_mr_slice_band_to_triggered, atomicv_mr_slice_bxor_to_triggered
    // );

    // gen_atomic_mr_op_def!((), (
    //     self,
    //     ioc: &[crate::iovec::Ioc<bool>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     dest_slice: &RemoteMemAddrSliceMut<bool>,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion,
    //     (
    //         ioc,
    //         desc,
    //         dest_addr,
    //         dest_slice.mem_address(),
    //         dest_slice.key(),
    //         context
    //     ),
    //     atomicv_lor_to_triggered, atomicv_land_to_triggered, atomicv_lxor_to_triggered,, atomicv_mr_slice_lor_to_triggered, atomicv_mr_slice_land_to_triggered, atomicv_mr_slice_lxor_to_triggered
    // );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<T>
    ) -> (),
        (
            buf,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_min_to_async, atomic_inject_max_to_async, atomic_inject_sum_to_async,  atomic_inject_prod_to_async, atomic_inject_bor_to_async, atomic_inject_band_to_async, atomic_inject_bxor_to_async,, atomic_inject_min_mr_slice_to_async, atomic_inject_max_mr_slice_to_async, atomic_inject_sum_mr_slice_to_async,  atomic_inject_prod_mr_slice_to_async, atomic_inject_bor_mr_slice_to_async, atomic_inject_band_mr_slice_to_async, atomic_inject_bxor_mr_slice_to_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        dest_addr: &crate::MappedAddress,
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ) -> (),
        (
            buf,
            dest_addr,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_lor_to_async, atomic_inject_land_to_async, atomic_inject_lxor_to_async,, atomic_inject_lor_mr_slice_to_async, atomic_inject_land_mr_slice_to_async, atomic_inject_lxor_mr_slice_to_async
    );

    // #[allow(clippy::too_many_arguments)]
    // unsafe fn atomic_slice_to_async<T: AsFiType>(
    //     &self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc>,
    //     dest_addr: &crate::MappedAddress,
    //     dst_slice: &RemoteMemAddrSliceMut<T>,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
    //     self.atomic_to_async(
    //         buf,
    //         desc,
    //         dest_addr,
    //         dst_slice.mem_address(),
    //         &dst_slice.key(),
    //         op,
    //         context,
    //     )
    // }

    // #[allow(clippy::too_many_arguments)]
    // unsafe fn inject_atomic_slice_to_async<T: AsFiType>(
    //     &self,
    //     buf: &[T],
    //     dest_addr: &crate::MappedAddress,
    //     dst_slice: &RemoteMemAddrSliceMut<T>,
    //     op: crate::enums::AtomicOp,
    // ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
    //     assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
    //     self.inject_atomic_to_async(
    //         buf,
    //         dest_addr,
    //         dst_slice.mem_address(),
    //         &dst_slice.key(),
    //         op,
    //     )
    // }

    // #[allow(clippy::too_many_arguments)]
    // unsafe fn atomicv_slice_to_async<T: AsFiType>(
    //     &self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     dst_slice: &RemoteMemAddrSliceMut<T>,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     // assert!(dst_slice.mem_len() == crate::iovec::Ioc::total_len(ioc));
    //     self.atomicv_to_async(
    //         ioc,
    //         desc,
    //         dest_addr,
    //         dst_slice.mem_address(),
    //         &dst_slice.key(),
    //         op,
    //         context,
    //     )
    // }

    unsafe fn atomicmsg_slice_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomic<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_to_async(msg, options)
    }
}

impl<EP: AsyncAtomicWriteEp> AsyncAtomicWriteRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp: ConnectedAsyncAtomicWriteEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion,   
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_min_async, atomic_max_async, atomic_sum_async,  atomic_prod_async, atomic_bor_async, atomic_band_async, atomic_bxor_async,, atomic_min_mr_slice_async, atomic_max_mr_slice_async, atomic_sum_mr_slice_async,  atomic_prod_mr_slice_async, atomic_bor_mr_slice_async, atomic_band_mr_slice_async, atomic_bxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomic_lor_async, atomic_land_async, atomic_lxor_async,, atomic_lor_mr_slice_async, atomic_land_mr_slice_async, atomic_lxor_mr_slice_async
    );

    // gen_atomic_mr_op_def!((<T: AsFiType>), (
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     dest_slice: &RemoteMemAddrSliceMut<T>,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion,
    //     (
    //         buf,
    //         desc,
    //         dest_addr,
    //         dest_slice.mem_address(),
    //         dest_slice.key(),
    //         context
    //     ),
    //     atomic_min_to_triggered, atomic_max_to_triggered, atomic_sum_to_triggered,  atomic_prod_to_triggered, atomic_bor_to_triggered, atomic_band_to_triggered, atomic_bxor_to_triggered,, atomic_mr_slice_min_to_triggered, atomic_mr_slice_max_to_triggered, atomic_mr_slice_sum_to_triggered,  atomic_mr_slice_prod_to_triggered, atomic_mr_slice_bor_to_triggered, atomic_mr_slice_band_to_triggered, atomic_mr_slice_bxor_to_triggered
    // );

    // gen_atomic_mr_op_def!((), (
    //     self,
    //     buf: &[bool],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     dest_slice: &RemoteMemAddrSliceMut<bool>,
    //     context: &mut TriggeredContext
    // ) -> SingleCompletion,   
    //     (
    //         buf,
    //         desc,
    //         dest_addr,
    //         dest_slice.mem_address(),
    //         dest_slice.key(),
    //         context
    //     ),
    //     atomic_lor_to_triggered, atomic_land_to_triggered, atomic_lxor_to_triggered,, atomic_mr_slice_lor_to_triggered, atomic_mr_slice_land_to_triggered, atomic_mr_slice_lxor_to_triggered
    // );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion, 
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_min_async, atomicv_max_async, atomicv_sum_async,  atomicv_prod_async, atomicv_bor_async, atomicv_band_async, atomicv_bxor_async,, atomicv_min_mr_slice_async, atomicv_max_mr_slice_async, atomicv_sum_mr_slice_async,  atomicv_prod_mr_slice_async, atomicv_bor_mr_slice_async, atomicv_band_mr_slice_async, atomicv_bxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_slice: &RemoteMemAddrSliceMut<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            dest_slice.mem_address(),
            dest_slice.key(),
            context
        ),
        atomicv_lor_async, atomicv_land_async, atomicv_lxor_async,, atomicv_lor_mr_slice_async, atomicv_land_mr_slice_async, atomicv_lxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        dest_slice: &RemoteMemAddrSliceMut<T>
    ) -> (),
        (
            buf,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_min_async, atomic_inject_max_async, atomic_inject_sum_async,  atomic_inject_prod_async, atomic_inject_bor_async, atomic_inject_band_async, atomic_inject_bxor_async,, atomic_inject_min_mr_slice_async, atomic_inject_max_mr_slice_async, atomic_inject_sum_mr_slice_async,  atomic_inject_prod_mr_slice_async, atomic_inject_bor_mr_slice_async, atomic_inject_band_mr_slice_async, atomic_inject_bxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        dest_slice: &RemoteMemAddrSliceMut<bool>
    ) -> (),
        (
            buf,
            dest_slice.mem_address(),
            dest_slice.key()
        ),
        atomic_inject_lor_async, atomic_inject_land_async, atomic_inject_lxor_async,, atomic_inject_lor_mr_slice_async, atomic_inject_land_mr_slice_async, atomic_inject_lxor_mr_slice_async
    );


    // unsafe fn atomic_slice_async<T: AsFiType>(
    //     &self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc>,
    //     dst_slice: &RemoteMemAddrSliceMut<T>,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
    //     self.atomic_async(
    //         buf,
    //         desc,
    //         dst_slice.mem_address(),
    //         &dst_slice.key(),
    //         op,
    //         context,
    //     )
    // }

    // unsafe fn inject_atomic_slice_async<T: AsFiType>(
    //     &self,
    //     buf: &[T],
    //     dst_slice: &RemoteMemAddrSliceMut<T>,
    //     op: crate::enums::AtomicOp,
    // ) -> impl std::future::Future<Output = Result<(), crate::error::Error>> {
    //     assert!(dst_slice.mem_size() == std::mem::size_of_val(buf));
    //     self.inject_atomic_async(buf, dst_slice.mem_address(), &dst_slice.key(), op)
    // }

    // unsafe fn atomicv_slice_async<T: AsFiType>(
    //     &self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dst_slice: &RemoteMemAddrSliceMut<T>,
    //     op: crate::enums::AtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     // assert!(dst_slice.mem_len() == crate::iovec::Ioc::total_len(ioc));
    //     self.atomicv_async(
    //         ioc,
    //         desc,
    //         dst_slice.mem_address(),
    //         &dst_slice.key(),
    //         op,
    //         context,
    //     )
    // }

    unsafe fn atomicmsg_slice_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgAtomicConnected<T>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.atomicmsg_async(msg, options)
    }
}

impl<EP: ConnectedAsyncAtomicWriteEp> ConnectedAsyncAtomicWriteRemoteMemAddrSliceEp for EP {}

pub(crate) trait AsyncAtomicFetchEpImpl: AtomicFetchEpImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async fn fetch_atomic_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.fetch_atomic_impl(
                buf,
                desc,
                res,
                res_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
                op,
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_atomicv_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
        op: crate::enums::FetchAtomicOp,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.fetch_atomicv_impl(
                ioc,
                desc,
                resultv,
                res_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
                op,
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    async fn fetch_atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgFetchAtomic<'_, T>,
            &mut crate::msg::MsgFetchAtomicConnected<'_, T>,
        >,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.fetch_atomicmsg_impl(imm_msg.to_owned(), resultv, res_desc, options)
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicFetchEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomic_min_from_async, fetch_atomic_max_from_async, fetch_atomic_sum_from_async, fetch_atomic_prod_from_async, fetch_atomic_bor_from_async, fetch_atomic_band_from_async, fetch_atomic_bxor_from_async
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomic_lor_from_async, fetch_atomic_land_from_async, fetch_atomic_lxor_from_async
    );
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomicv_min_from_async, fetch_atomicv_max_from_async, fetch_atomicv_sum_from_async, fetch_atomicv_prod_from_async, fetch_atomicv_bor_from_async, fetch_atomicv_band_from_async, fetch_atomicv_bxor_from_async
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<bool>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomicv_lor_from_async, fetch_atomicv_land_from_async, fetch_atomicv_lxor_from_async
    );

    unsafe fn fetch_atomicmsg_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicFetchEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomic_min_async, fetch_atomic_max_async, fetch_atomic_sum_async, fetch_atomic_prod_async, fetch_atomic_bor_async, fetch_atomic_band_async, fetch_atomic_bxor_async
    );

    gen_atomic_op_decl!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomic_lor_async, fetch_atomic_land_async, fetch_atomic_lxor_async
    );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomicv_min_async, fetch_atomicv_max_async, fetch_atomicv_sum_async, fetch_atomicv_prod_async, fetch_atomicv_bor_async, fetch_atomicv_band_async, fetch_atomicv_bxor_async
    );

    gen_atomic_op_decl!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<bool>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )-> SingleCompletion,
    fetch_atomicv_lor_async, fetch_atomicv_land_async, fetch_atomicv_lxor_async
    );

    unsafe fn fetch_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

macro_rules! gen_atomic_fetch_mr {
    ($($func_name: ident,)+, $($inner_func_name: ident),+) => {
        $(
            unsafe fn $func_name<T: AsFiType, RT: AsFiType> (
                &self,
                mr_slice: &MemoryRegionSlice,
                res_mr_slice: &mut MemoryRegionSliceMut,
                dest_addr: &crate::MappedAddress,
                mem_addr: RemoteMemoryAddress<RT>,
                mapped_key: &MappedMemoryRegionKey,
                context: &mut Context,
            ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
                let result_desc = res_mr_slice.desc();

                self.$inner_func_name(
                    mr_slice.as_slice() ,
                    Some(mr_slice.desc()),
                    res_mr_slice.as_mut_slice() ,
                    Some(result_desc),
                    dest_addr,
                    mem_addr,
                    mapped_key,
                    context,
                )
            }
        )+

    }
}

macro_rules! gen_conn_atomic_fetch_mr {
    ($($func_name: ident,)+, $($inner_func_name: ident),+) => {
        $(
            unsafe fn $func_name<T: AsFiType, RT: AsFiType> (
                &self,
                mr_slice: &MemoryRegionSlice,
                res_mr_slice: &mut MemoryRegionSliceMut,
                mem_addr: RemoteMemoryAddress<RT>,
                mapped_key: &MappedMemoryRegionKey,
                context: &mut Context,
            ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
                let result_desc = res_mr_slice.desc();

                self.$inner_func_name(
                    mr_slice.as_slice() ,
                    Some(mr_slice.desc()),
                    res_mr_slice.as_mut_slice() ,
                    Some(result_desc),
                    mem_addr,
                    mapped_key,
                    context,
                )
            }
        )+

    }
}

pub trait AsyncAtomicFetchEpMrSlice: AsyncAtomicFetchEp {
    gen_atomic_fetch_mr!(
        fetch_atomic_min_mr_slice_from_async, fetch_atomic_max_mr_slice_from_async, fetch_atomic_sum_mr_slice_from_async, fetch_atomic_prod_mr_slice_from_async, fetch_atomic_bor_mr_slice_from_async, fetch_atomic_band_mr_slice_from_async, fetch_atomic_bxor_mr_slice_from_async,, 
        fetch_atomic_min_from_async, fetch_atomic_max_from_async, fetch_atomic_sum_from_async, fetch_atomic_prod_from_async, fetch_atomic_bor_from_async, fetch_atomic_band_from_async, fetch_atomic_bxor_from_async
    );
}

impl<EP: AsyncAtomicFetchEp> AsyncAtomicFetchEpMrSlice for EP {}

pub trait ConnectedAsyncAtomicFetchEpMrSlice: ConnectedAsyncAtomicFetchEp {
    gen_conn_atomic_fetch_mr!(
        fetch_atomic_min_mr_slice_async, fetch_atomic_max_mr_slice_async, fetch_atomic_sum_mr_slice_async, fetch_atomic_prod_mr_slice_async, fetch_atomic_bor_mr_slice_async, fetch_atomic_band_mr_slice_async, fetch_atomic_bxor_mr_slice_async,, 
        fetch_atomic_min_async, fetch_atomic_max_async, fetch_atomic_sum_async, fetch_atomic_prod_async, fetch_atomic_bor_async, fetch_atomic_band_async, fetch_atomic_bxor_async
    );
}

impl<EP: ConnectedAsyncAtomicFetchEp> ConnectedAsyncAtomicFetchEpMrSlice for EP {}

impl<E: AsyncAtomicFetchEpImpl> AsyncAtomicFetchEpImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicFetchEpImpl> AsyncAtomicFetchEpImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicFetchEpImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + ReadMod, STATE: EpState> AsyncAtomicFetchEpImpl for TxContextImpl<I, STATE> {}

impl<I: AtomicCap + ReadMod, STATE: EpState> AsyncAtomicFetchEpImpl for TxContext<I, STATE> {}

impl<EP: AsyncAtomicFetchEpImpl + ConnlessEp> AsyncAtomicFetchEp for EP {
    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        fetch_atomic_async_impl(buf, desc, res, res_desc, Some(dest_addr), mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Min, crate::enums::FetchAtomicOp::Max, crate::enums::FetchAtomicOp::Sum, crate::enums::FetchAtomicOp::Prod, crate::enums::FetchAtomicOp::Bor, crate::enums::FetchAtomicOp::Band, crate::enums::FetchAtomicOp::Bxor,,
        fetch_atomic_min_from_async, fetch_atomic_max_from_async, fetch_atomic_sum_from_async, fetch_atomic_prod_from_async, fetch_atomic_bor_from_async, fetch_atomic_band_from_async, fetch_atomic_bxor_from_async
    );

    gen_atomic_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        fetch_atomic_async_impl(buf, desc, res, res_desc, Some(dest_addr), mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Lor, crate::enums::FetchAtomicOp::Land, crate::enums::FetchAtomicOp::Lxor,,
        fetch_atomic_lor_from_async, fetch_atomic_land_from_async, fetch_atomic_lxor_from_async
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        fetch_atomicv_async_impl(ioc, desc, resultv, res_desc, Some(dest_addr), mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Min, crate::enums::FetchAtomicOp::Max, crate::enums::FetchAtomicOp::Sum, crate::enums::FetchAtomicOp::Prod, crate::enums::FetchAtomicOp::Bor, crate::enums::FetchAtomicOp::Band, crate::enums::FetchAtomicOp::Bxor,,
        fetch_atomicv_min_from_async, fetch_atomicv_max_from_async, fetch_atomicv_sum_from_async, fetch_atomicv_prod_from_async, fetch_atomicv_bor_from_async, fetch_atomicv_band_from_async, fetch_atomicv_bxor_from_async
    );

    gen_atomic_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<bool>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        fetch_atomicv_async_impl(ioc, desc, resultv, res_desc, Some(dest_addr), mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Lor, crate::enums::FetchAtomicOp::Land, crate::enums::FetchAtomicOp::Lxor,,
        fetch_atomicv_lor_from_async, fetch_atomicv_land_from_async, fetch_atomicv_lxor_from_async
    );

    unsafe fn fetch_atomicmsg_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async_impl(Either::Left(msg), resultv, res_desc, options)
    }
}

impl<EP: AsyncAtomicFetchEpImpl + ConnectedEp> ConnectedAsyncAtomicFetchEp for EP {
    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        fetch_atomic_async_impl(buf, desc, res, res_desc, None, mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Min, crate::enums::FetchAtomicOp::Max, crate::enums::FetchAtomicOp::Sum, crate::enums::FetchAtomicOp::Prod, crate::enums::FetchAtomicOp::Bor, crate::enums::FetchAtomicOp::Band, crate::enums::FetchAtomicOp::Bxor,,
        fetch_atomic_min_async, fetch_atomic_max_async, fetch_atomic_sum_async, fetch_atomic_prod_async, fetch_atomic_bor_async, fetch_atomic_band_async, fetch_atomic_bxor_async
    );

    gen_atomic_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc<'_>>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    )->SingleCompletion,
        fetch_atomic_async_impl(buf, desc, res, res_desc, None, mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Lor, crate::enums::FetchAtomicOp::Land, crate::enums::FetchAtomicOp::Lxor,,
        fetch_atomic_lor_async, fetch_atomic_land_async, fetch_atomic_lxor_async
    );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context

    )->SingleCompletion,
        fetch_atomicv_async_impl(
            ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Min, crate::enums::FetchAtomicOp::Max, crate::enums::FetchAtomicOp::Sum, crate::enums::FetchAtomicOp::Prod, crate::enums::FetchAtomicOp::Bor, crate::enums::FetchAtomicOp::Band, crate::enums::FetchAtomicOp::Bxor,,
        fetch_atomicv_min_async, fetch_atomicv_max_async, fetch_atomicv_sum_async, fetch_atomicv_prod_async, fetch_atomicv_bor_async, fetch_atomicv_band_async, fetch_atomicv_bxor_async
    );

    gen_atomic_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<bool>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<bool>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context

    )->SingleCompletion,
        fetch_atomicv_async_impl(
            ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, context),
        crate::enums::FetchAtomicOp::Lor, crate::enums::FetchAtomicOp::Land, crate::enums::FetchAtomicOp::Lxor,,
        fetch_atomicv_lor_async, fetch_atomicv_land_async, fetch_atomicv_lxor_async
    );

    unsafe fn fetch_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async_impl(Either::Right(msg), resultv, res_desc, options)
    }
}

pub trait AsyncAtomicFetchRemoteMemAddrSliceEp: AsyncAtomicFetchEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            res,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomic_min_from_async, fetch_atomic_max_from_async, fetch_atomic_sum_from_async, fetch_atomic_prod_from_async, fetch_atomic_bor_from_async, fetch_atomic_band_from_async, fetch_atomic_bxor_from_async,,
        fetch_atomic_min_mr_slice_from_async, fetch_atomic_max_mr_slice_from_async, fetch_atomic_sum_mr_slice_from_async, fetch_atomic_prod_mr_slice_from_async, fetch_atomic_bor_mr_slice_from_async, fetch_atomic_band_mr_slice_from_async, fetch_atomic_bxor_mr_slice_from_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            res,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomic_lor_from_async, fetch_atomic_land_from_async, fetch_atomic_lxor_from_async,,
        fetch_atomic_lor_mr_slice_from_async, fetch_atomic_land_mr_slice_from_async, fetch_atomic_lxor_mr_slice_from_async
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            resultv,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomicv_min_from_async, fetch_atomicv_max_from_async, fetch_atomicv_sum_from_async, fetch_atomicv_prod_from_async, fetch_atomicv_bor_from_async, fetch_atomicv_band_from_async, fetch_atomicv_bxor_from_async,,
        fetch_atomicv_min_mr_slice_from_async, fetch_atomicv_max_mr_slice_from_async, fetch_atomicv_sum_mr_slice_from_async, fetch_atomicv_prod_mr_slice_from_async, fetch_atomicv_bor_mr_slice_from_async, fetch_atomicv_band_mr_slice_from_async, fetch_atomicv_bxor_mr_slice_from_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<bool>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        src_slice: &RemoteMemAddrSlice<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            resultv,
            res_desc,
            dest_addr,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomicv_lor_from_async, fetch_atomicv_land_from_async, fetch_atomicv_lxor_from_async,,
        fetch_atomicvmr_slice_lor_from_async, fetch_atomicv_land_mr_slice_from_async, fetch_atomicv_lxor_mr_slice_from_async
    );

    unsafe fn fetch_atomicmsg_slice_from_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomic<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_from_async(msg, resultv, res_desc, options)
    }
}

impl<EP: AsyncAtomicFetchEp> AsyncAtomicFetchRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp: ConnectedAsyncAtomicFetchEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc>,
        res: &mut [T],
        res_desc: Option<MemoryRegionDesc<'_>>,
        src_slice: &RemoteMemAddrSlice<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            res,
            res_desc,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomic_min_async, fetch_atomic_max_async, fetch_atomic_sum_async, fetch_atomic_prod_async, fetch_atomic_bor_async, fetch_atomic_band_async, fetch_atomic_bxor_async,,
        fetch_atomic_min_mr_slice_async, fetch_atomic_max_mr_slice_async, fetch_atomic_sum_mr_slice_async, fetch_atomic_prod_mr_slice_async, fetch_atomic_bor_mr_slice_async, fetch_atomic_band_mr_slice_async, fetch_atomic_bxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        buf: &[bool],
        desc: Option<MemoryRegionDesc>,
        res: &mut [bool],
        res_desc: Option<MemoryRegionDesc<'_>>,
        src_slice: &RemoteMemAddrSlice<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            res,
            res_desc,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomic_lor_async, fetch_atomic_land_async, fetch_atomic_lxor_async,,
        fetch_atomicmr_slice_lor_async, fetch_atomic_land_mr_slice_async, fetch_atomic_lxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        src_slice: &RemoteMemAddrSlice<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            resultv,
            res_desc,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomicv_min_async, fetch_atomicv_max_async, fetch_atomicv_sum_async, fetch_atomicv_prod_async, fetch_atomicv_bor_async, fetch_atomicv_band_async, fetch_atomicv_bxor_async,,
        fetch_atomicv_min_mr_slice_async, fetch_atomicv_max_mr_slice_async, fetch_atomicv_sum_mr_slice_async, fetch_atomicv_prod_mr_slice_async, fetch_atomicv_bor_mr_slice_async, fetch_atomicv_band_mr_slice_async, fetch_atomicv_bxor_mr_slice_async
    );

    gen_atomic_mr_op_def!((), (
        self,
        ioc: &[crate::iovec::Ioc<bool>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<bool>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        src_slice: &RemoteMemAddrSlice<bool>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            resultv,
            res_desc,
            src_slice.mem_address(),
            &src_slice.key(),
            context
        ),
        fetch_atomicv_lor_async, fetch_atomicv_land_async, fetch_atomicv_lxor_async,,
        fetch_atomicv_lor_mr_slice_async, fetch_atomicv_land_mr_slice_async, fetch_atomicv_lxor_mr_slice_async
    );

    unsafe fn fetch_atomicmsg_slice_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgFetchAtomicConnected<T>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicFetchMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.fetch_atomicmsg_async(msg, resultv, res_desc, options)
    }
}

impl<EP: ConnectedAsyncAtomicFetchEp> ConnectedAsyncAtomicFetchRemoteMemAddrSliceEp for EP {}

pub(crate) trait AsyncAtomicCASImpl: AtomicCASImpl + AsyncTxEp {
    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomic_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.compare_atomic_impl(
                buf,
                desc,
                compare,
                compare_desc,
                result,
                result_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
                op,
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomicv_async_impl<T: AsFiOrBoolType, RT: AsFiOrBoolType>(
        &self,
        ioc: &[crate::iovec::Ioc<'_, T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<'_, T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        ctx: &mut Context,
        op: crate::enums::CompareAtomicOp,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.compare_atomicv_impl(
                ioc,
                desc,
                comparetv,
                compare_desc,
                resultv,
                res_desc,
                dest_addr,
                mem_addr,
                mapped_key,
                Some(ctx.inner_mut()),
                op,
            )
        })
        .await?;
        cq.wait_for_ctx_async(ctx).await
    }

    #[allow(clippy::too_many_arguments)]
    async unsafe fn compare_atomicmsg_async_impl<T: AsFiType>(
        &self,
        mut msg: Either<
            &mut crate::msg::MsgCompareAtomic<'_, T>,
            &mut crate::msg::MsgCompareAtomicConnected<'_, T>,
        >,
        comparev: &[crate::iovec::Ioc<'_, T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<'_, T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> Result<SingleCompletion, crate::error::Error> {
        let imm_msg = match &msg {
            Either::Left(msg) => Either::Left(&**msg),
            Either::Right(msg) => Either::Right(&**msg),
        };

        let cq = self.retrieve_tx_cq();
        while_try_again(cq.as_ref(), || {
            self.compare_atomicmsg_impl(
                imm_msg.to_owned(),
                comparev,
                compare_desc,
                resultv,
                res_desc,
                options,
            )
        })
        .await?;

        let ctx = match &mut msg {
            Either::Left(msg) => msg.context(),
            Either::Right(msg) => msg.context(),
        };

        cq.wait_for_ctx_async(ctx).await
    }
}

pub trait AsyncAtomicCASEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
    compare_atomic_swap_to_async, compare_atomic_swap_ne_to_async, compare_atomic_swap_le_to_async, compare_atomic_swap_lt_to_async, compare_atomic_swap_ge_to_async, compare_atomic_swap_gt_to_async, compare_atomic_mswap_to_async
    );

    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    // (
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     compare: &[T],
    //     compare_desc: Option<MemoryRegionDesc<'_>>,
    //     result: &mut [T],
    //     result_desc: Option<MemoryRegionDesc<'_>>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // compare_atomic_swap_to_triggered, compare_atomic_swap_ne_to_triggered, compare_atomic_swap_le_to_triggered, compare_atomic_swap_lt_to_triggered, compare_atomic_swap_ge_to_triggered, compare_atomic_swap_gt_to_triggered, compare_atomic_mswap_to_triggered
    // );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
    compare_atomicv_swap_to_async, compare_atomicv_swap_ne_to_async, compare_atomicv_swap_le_to_async, compare_atomicv_swap_lt_to_async, compare_atomicv_swap_ge_to_async, compare_atomicv_swap_gt_to_async, compare_atomicv_mswap_to_async
    );

    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    // (
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     comparetv: &[crate::iovec::Ioc<T>],
    //     compare_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     resultv: &mut [crate::iovec::IocMut<T>],
    //     res_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // compare_atomicv_swap_to_triggered, compare_atomicv_swap_ne_to_triggered, compare_atomicv_swap_le_to_triggered, compare_atomicv_swap_lt_to_triggered, compare_atomicv_swap_ge_to_triggered, compare_atomicv_swap_gt_to_triggered, compare_atomicv_mswap_to_triggered
    // );

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

pub trait ConnectedAsyncAtomicCASEp {
    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc<'_>>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc<'_>>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
    compare_atomic_swap_async, compare_atomic_swap_ne_async, compare_atomic_swap_le_async, compare_atomic_swap_lt_async, compare_atomic_swap_ge_async, compare_atomic_swap_gt_async, compare_atomic_mswap_async
    );

    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    // (
    //     self,
    //     buf: &[T],
    //     desc: Option<MemoryRegionDesc<'_>>,
    //     compare: &[T],
    //     compare_desc: Option<MemoryRegionDesc<'_>>,
    //     result: &mut [T],
    //     result_desc: Option<MemoryRegionDesc<'_>>,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // compare_atomic_swap_triggered, compare_atomic_swap_ne_triggered, compare_atomic_swap_le_triggered, compare_atomic_swap_lt_triggered, compare_atomic_swap_ge_triggered, compare_atomic_swap_gt_triggered, compare_atomic_mswap_triggered
    // );

    gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
    compare_atomicv_swap_async, compare_atomicv_swap_ne_async, compare_atomicv_swap_le_async, compare_atomicv_swap_lt_async, compare_atomicv_swap_ge_async, compare_atomicv_swap_gt_async, compare_atomicv_mswap_async
    );

    // gen_atomic_op_decl!((<T: AsFiType, RT: AsFiType>), 
    // (
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     comparetv: &[crate::iovec::Ioc<T>],
    //     compare_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     resultv: &mut [crate::iovec::IocMut<T>],
    //     res_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    // compare_atomicv_swap_triggered, compare_atomicv_swap_ne_triggered, compare_atomicv_swap_le_triggered, compare_atomicv_swap_lt_triggered, compare_atomicv_swap_ge_triggered, compare_atomicv_swap_gt_triggered, compare_atomicv_mswap_triggered
    // );

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>>;
}

macro_rules! gen_atomic_cas_mr {
    ($($func_name: ident,)+, $($inner_func_name: ident),+) => {
        $(
            unsafe fn $func_name<T: AsFiType, RT: AsFiType> (
                &self,
                mr_slice: &MemoryRegionSlice,
                compare_mr_slice: &MemoryRegionSlice,
                result_mr_slice: &mut MemoryRegionSliceMut,
                dest_addr: &crate::MappedAddress,
                mem_addr: RemoteMemoryAddress<RT>,
                mapped_key: &MappedMemoryRegionKey,
                context: &mut Context,
            ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
                let result_desc = result_mr_slice.desc();

                self.$inner_func_name(
                    mr_slice.as_slice() ,
                    Some(mr_slice.desc()),
                    compare_mr_slice.as_slice() ,
                    Some(compare_mr_slice.desc()),
                    result_mr_slice.as_mut_slice() ,
                    Some(result_desc),
                    dest_addr,
                    mem_addr,
                    mapped_key,
                    context,
                )
            }
        )+

    }
}

macro_rules! gen_conn_atomic_cas_mr {
    ($($func_name: ident,)+, $($inner_func_name: ident),+) => {
        $(
            unsafe fn $func_name<T: AsFiType, RT: AsFiType> (
                &self,
                mr_slice: &MemoryRegionSlice,
                compare_mr_slice: &MemoryRegionSlice,
                result_mr_slice: &mut MemoryRegionSliceMut,
                mem_addr: RemoteMemoryAddress<RT>,
                mapped_key: &MappedMemoryRegionKey,
                context: &mut Context,
            ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
                let result_desc = result_mr_slice.desc();

                self.$inner_func_name(
                    mr_slice.as_slice() ,
                    Some(mr_slice.desc()),
                    compare_mr_slice.as_slice() ,
                    Some(compare_mr_slice.desc()),
                    result_mr_slice.as_mut_slice() ,
                    Some(result_desc),
                    mem_addr,
                    mapped_key,
                    context,
                )
            }
        )+

    }
}

pub trait AsyncAtomicCASEpMrSlice: AsyncAtomicCASEp {
    gen_atomic_cas_mr!(
        compare_atomic_swap_mr_to_async, compare_atomic_swap_ne_mr_to_async, compare_atomic_swap_le_mr_to_async, compare_atomic_swap_lt_mr_to_async, compare_atomic_swap_ge_mr_to_async, compare_atomic_swap_gt_mr_to_async, compare_atomic_mswap_mr_to_async,,
        compare_atomic_swap_to_async, compare_atomic_swap_ne_to_async, compare_atomic_swap_le_to_async, compare_atomic_swap_lt_to_async, compare_atomic_swap_ge_to_async, compare_atomic_swap_gt_to_async, compare_atomic_mswap_to_async
    );
    // #[allow(clippy::too_many_arguments)]
    // unsafe fn compare_atomic_mr_slice_to_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     mr_slice: &MemoryRegionSlice,
    //     compare_mr_slice: &MemoryRegionSlice,
    //     result_mr_slice: &mut MemoryRegionSliceMut,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::CompareAtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     let result_desc = result_mr_slice.desc();
    //     self.compare_atomic_to_async(
    //         mr_slice.as_slice() ,
    //         Some(mr_slice.desc()),
    //         compare_mr_slice.as_slice() ,
    //         Some(compare_mr_slice.desc()),
    //         result_mr_slice.as_mut_slice() ,
    //         Some(result_desc),
    //         dest_addr,
    //         mem_addr,
    //         mapped_key,
    //         context,
    //         op,
    //     )
    // }
}

impl<EP: AsyncAtomicCASEp> AsyncAtomicCASEpMrSlice for EP {}

pub trait ConnectedAsyncAtomicCASEpMrSlice: ConnectedAsyncAtomicCASEp {
    gen_conn_atomic_cas_mr!(
        compare_atomic_swap_mr_async, compare_atomic_swap_ne_mr_async, compare_atomic_swap_le_mr_async, compare_atomic_swap_lt_mr_async, compare_atomic_swap_ge_mr_async, compare_atomic_swap_gt_mr_async, compare_atomic_mswap_mr_async,,
        compare_atomic_swap_async, compare_atomic_swap_ne_async, compare_atomic_swap_le_async, compare_atomic_swap_lt_async, compare_atomic_swap_ge_async, compare_atomic_swap_gt_async, compare_atomic_mswap_async
    );
    // #[allow(clippy::too_many_arguments)]
    // unsafe fn compare_atomic_mr_slice_async<T: AsFiType, RT: AsFiType>(
    //     &self,
    //     mr_slice: &MemoryRegionSlice,
    //     compare_mr_slice: &MemoryRegionSlice,
    //     result_mr_slice: &mut MemoryRegionSliceMut,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     op: crate::enums::CompareAtomicOp,
    //     context: &mut Context,
    // ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
    //     let result_desc = result_mr_slice.desc();
    //     self.compare_atomic_async(
    //         mr_slice.as_slice() ,
    //         Some(mr_slice.desc()),
    //         compare_mr_slice.as_slice() ,
    //         Some(compare_mr_slice.desc()),
    //         result_mr_slice.as_mut_slice() ,
    //         Some(result_desc),
    //         mem_addr,
    //         mapped_key,
    //         op,
    //         context,
    //     )
    // }
}

impl<EP: ConnectedAsyncAtomicCASEp> ConnectedAsyncAtomicCASEpMrSlice for EP {}

impl<E: AsyncAtomicCASImpl> AsyncAtomicCASImpl for EndpointBase<E, Connected> {}
impl<E: AsyncAtomicCASImpl> AsyncAtomicCASImpl for EndpointBase<E, Connectionless> {}

impl<EP: AtomicCap + WriteMod + ReadMod, EQ: ?Sized + AsyncReadEq, CQ: AsyncReadCq + ?Sized>
    AsyncAtomicCASImpl for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: AtomicCap + WriteMod + ReadMod, STATE: EpState> AsyncAtomicCASImpl
    for TxContextImpl<I, STATE>
{
}

impl<I: AtomicCap + WriteMod + ReadMod, STATE: EpState> AsyncAtomicCASImpl for TxContext<I, STATE> {}

impl<EP: AsyncAtomicCASImpl + ConnlessEp> AsyncAtomicCASEp for EP {

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    (
            self,
            buf: &[T],
            desc: Option<MemoryRegionDesc<'_>>,
            compare: &[T],
            compare_desc: Option<MemoryRegionDesc<'_>>,
            result: &mut [T],
            result_desc: Option<MemoryRegionDesc<'_>>,
            dest_addr: &crate::MappedAddress,
            mem_addr: RemoteMemoryAddress<RT>,
            mapped_key: &MappedMemoryRegionKey,
            context: &mut Context
    ) -> SingleCompletion,

        compare_atomic_async_impl
        (
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            context
        ), crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
        compare_atomic_swap_to_async, compare_atomic_swap_ne_to_async, compare_atomic_swap_le_to_async, compare_atomic_swap_lt_to_async, compare_atomic_swap_ge_to_async, compare_atomic_swap_gt_to_async, compare_atomic_mswap_to_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    // (
    //         self,
    //         buf: &[T],
    //         desc: Option<MemoryRegionDesc<'_>>,
    //         compare: &[T],
    //         compare_desc: Option<MemoryRegionDesc<'_>>,
    //         result: &mut [T],
    //         result_desc: Option<MemoryRegionDesc<'_>>,
    //         dest_addr: &crate::MappedAddress,
    //         mem_addr: RemoteMemoryAddress<RT>,
    //         mapped_key: &MappedMemoryRegionKey,
    //         context: &mut TriggeredContext
    // )-> SingleCompletion,

    //     compare_atomic_async_impl(
    //         buf,
    //         desc,
    //         compare,
    //         compare_desc,
    //         result,
    //         result_desc,
    //         Some(dest_addr),
    //         mem_addr,
    //         mapped_key,
    //         Some(context.inner_mut())
    //     ), crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
    //     compare_atomic_swap_to_triggered, compare_atomic_swap_ne_to_triggered, compare_atomic_swap_le_to_triggered, compare_atomic_swap_lt_to_triggered, compare_atomic_swap_ge_to_triggered, compare_atomic_swap_gt_to_triggered, compare_atomic_mswap_to_triggered
    // );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
        compare_atomicv_async_impl
        (
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            context
        ),
        crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
        compare_atomicv_swap_to_async, compare_atomicv_swap_ne_to_async, compare_atomicv_swap_le_to_async, compare_atomicv_swap_lt_to_async, compare_atomicv_swap_ge_to_async, compare_atomicv_swap_gt_to_async, compare_atomicv_mswap_to_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    // (
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     comparetv: &[crate::iovec::Ioc<T>],
    //     compare_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     resultv: &mut [crate::iovec::IocMut<T>],
    //     res_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    //     compare_atomicv_impl(
    //         ioc,
    //         desc,
    //         comparetv,
    //         compare_desc,
    //         resultv,
    //         res_desc,
    //         Some(dest_addr),
    //         mem_addr,
    //         mapped_key,
    //         Some(context.inner_mut())
    //     ),
    //     crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
    //     compare_atomicv_swap_to_triggered, compare_atomicv_swap_ne_to_triggered, compare_atomicv_swap_le_to_triggered, compare_atomicv_swap_lt_to_triggered, compare_atomicv_swap_ge_to_triggered, compare_atomicv_swap_gt_to_triggered, compare_atomicv_mswap_to_triggered
    // );

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async_impl(
            Either::Left(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
        )
    }
}

impl<EP: AsyncAtomicCASImpl + ConnectedEp> ConnectedAsyncAtomicCASEp for EP {

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    (
            self,
            buf: &[T],
            desc: Option<MemoryRegionDesc<'_>>,
            compare: &[T],
            compare_desc: Option<MemoryRegionDesc<'_>>,
            result: &mut [T],
            result_desc: Option<MemoryRegionDesc<'_>>,
            mem_addr: RemoteMemoryAddress<RT>,
            mapped_key: &MappedMemoryRegionKey,
            context: &mut Context
    ) -> SingleCompletion,

        compare_atomic_async_impl(
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            None,
            mem_addr,
            mapped_key,
            context
        ), crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
        compare_atomic_swap_async, compare_atomic_swap_ne_async, compare_atomic_swap_le_async, compare_atomic_swap_lt_async, compare_atomic_swap_ge_async, compare_atomic_swap_gt_async, compare_atomic_mswap_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    // (
    //         self,
    //         buf: &[T],
    //         desc: Option<MemoryRegionDesc<'_>>,
    //         compare: &[T],
    //         compare_desc: Option<MemoryRegionDesc<'_>>,
    //         result: &mut [T],
    //         result_desc: Option<MemoryRegionDesc<'_>>,
    //         dest_addr: &crate::MappedAddress,
    //         mem_addr: RemoteMemoryAddress<RT>,
    //         mapped_key: &MappedMemoryRegionKey,
    //         context: &mut TriggeredContext
    // )-> SingleCompletion,

    //     compare_atomic_async_impl(
    //         buf,
    //         desc,
    //         compare,
    //         compare_desc,
    //         result,
    //         result_desc,
    //         Some(dest_addr),
    //         mem_addr,
    //         mapped_key,
    //         Some(context.inner_mut())
    //     ), crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
    //     compare_atomic_swap_to_triggered, compare_atomic_swap_ne_to_triggered, compare_atomic_swap_le_to_triggered, compare_atomic_swap_lt_to_triggered, compare_atomic_swap_ge_to_triggered, compare_atomic_swap_gt_to_triggered, compare_atomic_mswap_to_triggered
    // );

    gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress<RT>,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context
    ) -> SingleCompletion,
        compare_atomicv_async_impl(
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            None,
            mem_addr,
            mapped_key,
            context
        ),
        crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
        compare_atomicv_swap_async, compare_atomicv_swap_ne_async, compare_atomicv_swap_le_async, compare_atomicv_swap_lt_async, compare_atomicv_swap_ge_async, compare_atomicv_swap_gt_async, compare_atomicv_mswap_async
    );

    // gen_atomic_op_def!((<T: AsFiType, RT: AsFiType>), 
    // (
    //     self,
    //     ioc: &[crate::iovec::Ioc<T>],
    //     desc: Option<&[MemoryRegionDesc<'_>]>,
    //     comparetv: &[crate::iovec::Ioc<T>],
    //     compare_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     resultv: &mut [crate::iovec::IocMut<T>],
    //     res_desc: Option<&[MemoryRegionDesc<'_>]>,
    //     dest_addr: &crate::MappedAddress,
    //     mem_addr: RemoteMemoryAddress<RT>,
    //     mapped_key: &MappedMemoryRegionKey,
    //     context: &mut TriggeredContext
    // ),
    //     compare_atomicv_impl(
    //         ioc,
    //         desc,
    //         comparetv,
    //         compare_desc,
    //         resultv,
    //         res_desc,
    //         Some(dest_addr),
    //         mem_addr,
    //         mapped_key,
    //         Some(context.inner_mut())
    //     ),
    //     crate::enums::CompareAtomicOp::Cswap, crate::enums::CompareAtomicOp::CswapNe, crate::enums::CompareAtomicOp::CswapLe, crate::enums::CompareAtomicOp::CswapLt, crate::enums::CompareAtomicOp::CswapGe, crate::enums::CompareAtomicOp::CswapGt, crate::enums::CompareAtomicOp::Mswap ,,
    //     compare_atomicv_swap_to_triggered, compare_atomicv_swap_ne_to_triggered, compare_atomicv_swap_le_to_triggered, compare_atomicv_swap_lt_to_triggered, compare_atomicv_swap_ge_to_triggered, compare_atomicv_swap_gt_to_triggered, compare_atomicv_mswap_to_triggered
    // );

    #[inline]
    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async_impl(
            Either::Right(msg),
            comparev,
            compare_desc,
            resultv,
            res_desc,
            options,
        )
    }
}

pub trait AsyncAtomicCASRemoteMemAddrSliceEp: AsyncAtomicCASEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            context
        ),
        compare_atomic_swap_to_async, compare_atomic_swap_ne_to_async, compare_atomic_swap_le_to_async, compare_atomic_swap_lt_to_async, compare_atomic_swap_ge_to_async, compare_atomic_swap_gt_to_async, compare_atomic_mswap_to_async,,
        compare_atomic_swap_mr_slice_to_async, compare_atomic_swap_ne_mr_slice_to_async, compare_atomic_swap_le_mr_slice_to_async, compare_atomic_swap_lt_mr_slice_to_async, compare_atomic_swap_ge_mr_slice_to_async, compare_atomic_swap_gt_mr_slice_to_async, compare_atomic_mswap_mr_slice_to_async
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dest_addr,
            dst_slice.mem_address(),
            &dst_slice.key(),
            context
        ),
        compare_atomicv_swap_to_async, compare_atomicv_swap_ne_to_async, compare_atomicv_swap_le_to_async, compare_atomicv_swap_lt_to_async, compare_atomicv_swap_ge_to_async, compare_atomicv_swap_gt_to_async, compare_atomicv_mswap_to_async,,
        compare_atomicv_swap_mr_slice_to_async, compare_atomicv_swap_ne_mr_slice_to_async, compare_atomicv_swap_le_mr_slice_to_async, compare_atomicv_swap_lt_mr_slice_to_async, compare_atomicv_swap_ge_mr_slice_to_async, compare_atomicv_swap_gt_mr_slice_to_async, compare_atomicv_mswap_mr_slice_to_async
    );

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_slice_to_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomic<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_to_async(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}

impl<EP: AsyncAtomicCASEp> AsyncAtomicCASRemoteMemAddrSliceEp for EP {}

pub trait ConnectedAsyncAtomicCASRemoteMemAddrSliceEp: ConnectedAsyncAtomicCASEp {
    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        buf: &[T],
        desc: Option<MemoryRegionDesc>,
        compare: &[T],
        compare_desc: Option<MemoryRegionDesc>,
        result: &mut [T],
        result_desc: Option<MemoryRegionDesc<'_>>,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            buf,
            desc,
            compare,
            compare_desc,
            result,
            result_desc,
            dst_slice.mem_address(),
            &dst_slice.key(),
            context
        ),
        compare_atomic_swap_async, compare_atomic_swap_ne_async, compare_atomic_swap_le_async, compare_atomic_swap_lt_async, compare_atomic_swap_ge_async, compare_atomic_swap_gt_async, compare_atomic_mswap_async,,
        compare_atomic_swap_mr_slice_async, compare_atomic_swap_ne_mr_slice_async, compare_atomic_swap_le_mr_slice_async, compare_atomic_swap_lt_mr_slice_async, compare_atomic_swap_ge_mr_slice_async, compare_atomic_swap_gt_mr_slice_async, compare_atomic_mswap_mr_slice_async
    );

    gen_atomic_mr_op_def!((<T: AsFiType>), (
        self,
        ioc: &[crate::iovec::Ioc<T>],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        comparetv: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        dst_slice: &RemoteMemAddrSliceMut<T>,
        context: &mut Context
    ) -> SingleCompletion,
        (
            ioc,
            desc,
            comparetv,
            compare_desc,
            resultv,
            res_desc,
            dst_slice.mem_address(),
            &dst_slice.key(),
            context
        ),
        compare_atomicv_swap_async, compare_atomicv_swap_ne_async, compare_atomicv_swap_le_async, compare_atomicv_swap_lt_async, compare_atomicv_swap_ge_async, compare_atomicv_swap_gt_async, compare_atomicv_mswap_async,,
        compare_atomicv_swap_mr_slice_async, compare_atomicv_swap_ne_mr_slice_async, compare_atomicv_swap_le_mr_slice_async, compare_atomicv_swap_lt_mr_slice_async, compare_atomicv_swap_ge_mr_slice_async, compare_atomicv_swap_gt_mr_slice_async, compare_atomicv_mswap_mr_slice_async
    );

    #[allow(clippy::too_many_arguments)]
    unsafe fn compare_atomicmsg_slice_async<T: AsFiType + 'static>(
        &self,
        msg: &mut crate::msg::MsgCompareAtomicConnected<T>,
        comparev: &[crate::iovec::Ioc<T>],
        compare_desc: Option<&[MemoryRegionDesc<'_>]>,
        resultv: &mut [crate::iovec::IocMut<T>],
        res_desc: Option<&[MemoryRegionDesc<'_>]>,
        options: AtomicMsgOptions,
    ) -> impl std::future::Future<Output = Result<SingleCompletion, crate::error::Error>> {
        self.compare_atomicmsg_async(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}

impl<EP: ConnectedAsyncAtomicCASEp> ConnectedAsyncAtomicCASRemoteMemAddrSliceEp for EP {}
