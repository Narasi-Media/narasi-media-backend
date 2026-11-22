use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum DomainError {
    #[error("Saldo tidak mencukupi. Saldo saat ini: {current}, Total tagihan: {required}")]
    InsufficientBalance { current: i64, required: i64 },

    #[error("Jumlah pesanan ({quantity}) di luar batas ketentuan. Minimal: {min}, Maksimal: {max}")]
    InvalidOrderQuantity { quantity: u32, min: u32, max: u32 },

    #[error("Target URL atau username tidak boleh kosong")]
    EmptyOrderTarget,

    #[error("Layanan tidak aktif atau tidak ditemukan")]
    ServiceInactive,

    #[error("Layanan komentar memerlukan minimal satu baris komentar")]
    EmptyCommentsForCustomService,

    #[error("Pesanan dengan ID {id} tidak memenuhi syarat untuk refill/garansi")]
    RefillNotEligible { id: String },

    #[error("Nominal deposit minimal adalah Rp {min}. Nominal yang diminta: Rp {requested}")]
    DepositBelowMinimum { requested: i64, min: i64 },

    #[error("Deposit berstatus {current_status} tidak dapat disetujui atau ditolak")]
    InvalidDepositStateTransition { current_status: String },

    #[error("Entity dengan ID {0} tidak ditemukan")]
    NotFound(String),

    #[error("Validasi gagal: {0}")]
    ValidationError(String),

    #[error("Kesalahan infrastruktur database: {0}")]
    Infrastructure(String),
}
