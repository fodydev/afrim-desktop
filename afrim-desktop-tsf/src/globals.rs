//! Global state and GUID definitions for the afrim-desktop TSF IME.

#[cfg(target_os = "windows")]
use once_cell::sync::OnceCell;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HMODULE;
#[cfg(target_os = "windows")]
use windows::core::GUID;

/// CLSID for the afrim-desktop text service COM class.
///
/// Generated UUID: {A7E3F1B2-4C8D-4F2E-9A1B-6D3E5F7C8A90}
#[cfg(target_os = "windows")]
pub const CLSID_AFRIM_DESKTOP_TEXT_SERVICE: GUID =
    GUID::from_u128(0xC017_051D_BBBD_4186_9176_E6F7_D2E2_92C6);

/// GUID for the afrim-desktop input profile.
///
/// Generated UUID: {B8F4E2C3-5D9A-4E3F-8B2C-7E4F6A8D9B01}
#[cfg(target_os = "windows")]
pub const GUID_AFRIM_DESKTOP_PROFILE: GUID =
    GUID::from_u128(0x019D_2E8B_56DE_4F1F_B607_58CA_272E_960C);

/// GUID for the preedit display attribute (underline).
///
/// Generated UUID: {C9A5D3E4-6EAB-4F40-9C3D-8F5A7B9EAC12}
#[cfg(target_os = "windows")]
pub const GUID_DISPLAY_ATTRIBUTE_INPUT: GUID =
    GUID::from_u128(0xA357_5133_8D6F_4A64_8298_38A5_7F5C_B0FC);

/// GUID for the converted text display attribute (bold underline).
///
/// Generated UUID: {DAB6E4F5-7FBC-4051-AD4E-9A6B8CAFBD23}
#[cfg(target_os = "windows")]
pub const GUID_DISPLAY_ATTRIBUTE_CONVERTED: GUID =
    GUID::from_u128(0x1F1A_7DD5_EB68_4D12_BF4B_A91C_3ECD_12F1);

/// GUID for the PreservedKey ON/OFF toggle (Hankaku/Zenkaku).
///
/// Generated UUID: {EBC7F5A6-8D1E-4F51-BE3D-AF6C9DAE0B34}
#[cfg(target_os = "windows")]
pub const GUID_PRESERVED_KEY_ONOFF: GUID =
    GUID::from_u128(0x9D42_1CAF_B03D_447A_A60B_930F_8F14_4D4D);

/// GUID for the Ctrl+Space preserved key (IME toggle alternative).
///
/// For keyboards without Hankaku/Zenkaku key.
/// Generated UUID: {C3D4E5F6-7A8B-4C9D-AE1F-2B3C4D5E6F70}
#[cfg(target_os = "windows")]
pub const GUID_PRESERVED_KEY_CTRL_SPACE: GUID =
    GUID::from_u128(0x5AD7_2FE1_42C7_499B_A269_9641_80A8_20E6);

/// GUID for the Language Bar button item.
///
/// Generated UUID: {FCD8A6B7-9E2F-4062-CF4E-BA7DAEBF1C45}
#[cfg(target_os = "windows")]
pub const GUID_LANGBAR_ITEM_BUTTON: GUID =
    GUID::from_u128(0xED93_7561_0C2B_47CF_925C_61BD_3631_9281);

/// GUID for immersive (modern/UWP app) support capability.
///
/// Required for the IME to be loaded in Windows Store / UWP / modern apps.
/// {13A016DF-560B-46CD-947A-4C3AF1E0E35D}
#[cfg(target_os = "windows")]
pub const GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT: GUID =
    GUID::from_u128(0x13A0_16DF_560B_46CD_947A_4C3A_F1E0_E35D);

/// GUID for system tray support capability.
///
/// Required for proper system tray integration in modern Windows.
/// {25504FB4-7BAB-4BC1-9C69-CF81890F0EF5}
#[cfg(target_os = "windows")]
pub const GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT: GUID =
    GUID::from_u128(0x2550_4FB4_7BAB_4BC1_9C69_CF81_890F_0EF5);

/// GUID for secure mode capability.
///
/// Required for IME to receive key events in modern/UWP apps on Windows 8+.
/// Without this, the IME appears in the language bar but keys are not dispatched.
/// {49D2F9CE-1F5E-11D7-A6D3-00065B84435C}
#[cfg(target_os = "windows")]
pub const GUID_TFCAT_TIPCAP_SECUREMODE: GUID =
    GUID::from_u128(0x49D2_F9CE_1F5E_11D7_A6D3_0006_5B84_435C);

/// GUID for UI element enabled capability.
///
/// Required for proper UI element handling in modern apps.
/// {49D2F9CF-1F5E-11D7-A6D3-00065B84435C}
#[cfg(target_os = "windows")]
pub const GUID_TFCAT_TIPCAP_UIELEMENTENABLED: GUID =
    GUID::from_u128(0x49D2_F9CF_1F5E_11D7_A6D3_0006_5B84_435C);

/// Language the TSF profile is registered under (LANGID 0x0409 = en-US).
///
/// afrim datasets target many languages that have no Windows LANGID of their
/// own, so the profile is listed under English with the Latin keyboard.
pub const LANGID_PROFILE: u16 = 0x0409;

/// Newtype wrapper around HMODULE to implement Send + Sync.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub struct SyncHmodule(pub HMODULE);

#[cfg(target_os = "windows")]
unsafe impl Send for SyncHmodule {}
#[cfg(target_os = "windows")]
unsafe impl Sync for SyncHmodule {}

/// DLL module handle, set during DllMain.
#[cfg(target_os = "windows")]
pub static DLL_INSTANCE: OnceCell<SyncHmodule> = OnceCell::new();

/// Global reference count for COM objects (used by DllCanUnloadNow).
#[cfg(target_os = "windows")]
pub static DLL_REF_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Increment the global DLL reference count.
#[cfg(target_os = "windows")]
pub fn dll_add_ref() {
    DLL_REF_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Decrement the global DLL reference count.
#[cfg(target_os = "windows")]
pub fn dll_release() {
    // Guard against underflow: if count is already 0, do not subtract.
    // AtomicUsize underflow wraps to usize::MAX, which would prevent DLL unload forever.
    let prev = DLL_REF_COUNT.load(std::sync::atomic::Ordering::SeqCst);
    if prev == 0 {
        tracing::error!(
            "DLL reference count underflow detected — release without matching add_ref"
        );
        return;
    }
    DLL_REF_COUNT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
}

/// Check if the DLL can be unloaded (no outstanding references).
#[cfg(target_os = "windows")]
pub fn dll_can_unload() -> bool {
    DLL_REF_COUNT.load(std::sync::atomic::Ordering::SeqCst) == 0
}
