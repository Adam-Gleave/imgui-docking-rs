use bitflags::bitflags;

use crate::string::ImStr;
use crate::sys;
use crate::Direction;
use crate::sys::ImGuiWindowClass;

bitflags! {
    /// Configuration flags for docking
    #[repr(transparent)]
    pub struct DockNodeFlags: u32 {
        /// Keep all docked items even if not visible
        const KEEP_ALIVE_ONLY = sys::ImGuiDockNodeFlags_KeepAliveOnly;

        /// Disable docking in the central node
        const NO_DOCKING_IN_CENTRAL_NODE = sys::ImGuiDockNodeFlags_NoDockingInCentralNode;

        /// Draw the whole dockspace background
        const PASSTHRU_CENTRAL_NODE = sys::ImGuiDockNodeFlags_PassthruCentralNode;

        /// Disable further splitting of child dock nodes
        const NO_SPLIT = sys::ImGuiDockNodeFlags_NoSplit;

        /// Disable resizing of child dock nodes
        const NO_RESIZE = sys::ImGuiDockNodeFlags_NoResize;

        /// Automatically hide tab bar of docked nodes
        const AUTO_HIDE_TAB_BAR = sys::ImGuiDockNodeFlags_AutoHideTabBar;
    }
}

/// Dockspace data, the root will extend to cover the viewport at all times
#[derive(Clone, PartialEq)]
pub struct DockSpace<'a> {
    id: &'a ImStr,
    flags: DockNodeFlags,
    window_class: *const ImGuiWindowClass,
}

impl<'a> DockSpace<'a> {
    pub fn new(id: &'a ImStr) -> Self {
        let window_class;
        
        unsafe { window_class = sys::ImGuiWindowClass_ImGuiWindowClass() };
        
        let mut dockspace = Self {
            id,
            flags: DockNodeFlags::PASSTHRU_CENTRAL_NODE,
            window_class,
        };

        dockspace.flags.insert(DockNodeFlags::PASSTHRU_CENTRAL_NODE);
        dockspace
    }

    #[inline]
    pub fn flags(mut self, flags: DockNodeFlags) -> Self {
        self.flags = flags;
        
        self
    }

    pub fn over_viewport(self) -> Self {
        unsafe { sys::igDockSpaceOverViewport(sys::igGetMainViewport(), self.flags.bits() as i32, self.window_class) };
        
        self
    }
}

impl<'a> Drop for DockSpace<'a> {
    fn drop(&mut self) {
        unsafe { sys::ImGuiWindowClass_destroy(self.window_class as *mut _ ) }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct DockNode {
    id: sys::ImGuiID,
}

impl DockNode {
    pub fn new(id: sys::ImGuiID) -> Self {
        Self { id }
    }

    pub fn size(self, size: [f32; 2]) -> Self {
        unsafe { sys::igDockBuilderSetNodeSize(self.id, sys::ImVec2::from(size)) }

        self
    }

    pub fn position(self, position: [f32; 2]) -> Self {
        unsafe { sys::igDockBuilderSetNodePos(self.id, sys::ImVec2::from(position)) }

        self
    }

    pub fn dock_window(self, window_name: &ImStr) -> Self {
        unsafe { sys::igDockBuilderDockWindow(window_name.as_ptr(), self.id) }

        self
    }

    pub fn split<D: FnOnce(DockNode), O: FnOnce(DockNode)>(
        self,
        split_dir: Direction,
        size_ratio: f32,
        dir: D,
        opposite_dir: O,
    ) {
        let mut out_id_at_dir: sys::ImGuiID = 0;
        let mut out_id_at_opposite_dir: sys::ImGuiID = 0;

        unsafe {
            sys::igDockBuilderSplitNode(
                self.id,
                split_dir as i32,
                size_ratio,
                &mut out_id_at_dir,
                &mut out_id_at_opposite_dir,
            );
        }

        dir(DockNode::new(out_id_at_dir));
        opposite_dir(DockNode::new(out_id_at_opposite_dir));
    }
}

pub struct Dock {}

impl Dock {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build<F: FnOnce(DockNode)>(self, f: F) {
        let dock_id = unsafe { sys::igDockBuilderAddNode(0, sys::ImGuiDockNodeFlags_None as i32) };

        f(DockNode::new(dock_id));

        unsafe { sys::igDockBuilderFinish(dock_id) }
    }
}
