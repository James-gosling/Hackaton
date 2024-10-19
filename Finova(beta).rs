// Importar las dependencias necesarias del marco de trabajo de Substrate
use frame_support::{decl_module, decl_storage, dispatch, ensure};
use sp_runtime::traits::{Zero, One};
use pallet_timestamp::Pallet as TimestampPallet;

// Definimos un enum para representar los tipos de moneda que se pueden usar
#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum Currency<T: Config> {
    Native, // Moneda nativa
    Token(T::AccountId), // Token basado en una cuenta
}

// Definimos una estructura para representar un préstamo
#[derive(Clone, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct Borrow<T: Config> {
    amount: BalanceOf<T>, // Monto del préstamo
    collateral: BalanceOf<T>, // Colateral proporcionado
    borrow_time: T::BlockNumber, // Tiempo en que se tomó el préstamo
}

// Definimos el módulo del pallet
#[frame_support::pallet]
pub mod pallet {
    use super::*;

    // Configuración del pallet
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>; // Tipo de evento
        type Currency: Currency<Self>; // Tipo de moneda
    }

    // Definimos las constantes para los contratos
    const ORDENANTE_CONTRACT_ID: &[u8] = b"0x6f53dfc173b316b4a7d370403617429231d406462d0fcbd23fdb2f1ce15306f0";
    const RECEPTOR_CONTRACT_ID: &[u8] = b"0x1a6e3d6989db4fd42c4283fd69367fff78d4a6c08c88ebf3f927348489177e22";

    // Almacenamiento para las cuentas de los usuarios
    #[pallet::storage]
    #[pallet::getter(fn accounts)]
    pub(super) type Accounts<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Account<T>>;

    // Almacenamiento para los préstamos activos
    #[pallet::storage]
    #[pallet::getter(fn borrows)]
    pub(super) type Borrows<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Borrow<T>>;

    // Implementación de las funciones del pallet
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Función para depositar fondos en el contrato ordenante
        #[pallet::weight(10_000)]
        pub fn deposit_order(origin: OriginFor<T>, amount: BalanceOf<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?; // Aseguramos que el origen es un usuario firmado

            // Transferir los tokens desde la cuenta del usuario al contrato ordenante
            T::Currency::transfer(&who, ORDENANTE_CONTRACT_ID, amount)?;

            // Actualizar el estado del usuario
            let mut account = Self::accounts.get(&who).unwrap_or_default(); // Obtener la cuenta del usuario o crear una nueva
            account.balance += amount; // Actualizar el balance total
            account.available_balance += amount; // Actualizar el balance disponible
            Self::accounts.insert(&who, account); // Guardar la cuenta actualizada

            Ok(()) // Retornar éxito
        }

        // Función para depositar fondos en el contrato receptor
        #[pallet::weight(10_000)]
        pub fn deposit_receiver(origin: OriginFor<T>, amount: BalanceOf<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?; // Aseguramos que el origen es un usuario firmado

            // Transferir los tokens desde la cuenta del usuario al contrato receptor
            T::Currency::transfer(&who, RECEPTOR_CONTRACT_ID, amount)?;

            // Lógica similar a la anterior para actualizar el estado del usuario
            let mut account = Self::accounts.get(&who).unwrap_or_default(); // Obtener la cuenta del usuario o crear una nueva
            account.balance += amount; // Actualizar el balance total
            account.available_balance += amount; // Actualizar el balance disponible
            Self::accounts.insert(&who, account); // Guardar la cuenta actualizada

            Ok(()) // Retornar éxito
        }

        // Función para pedir prestado
        #[pallet::weight(10_000)]
        pub fn borrow(origin: OriginFor<T>, amount: BalanceOf<T>, collateral: BalanceOf<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?; // Aseguramos que el origen es un usuario firmado

            // Lógica para verificar si el usuario puede pedir prestado
            let mut account = Self::accounts.get(&who).ok_or(Error::<T>::AccountNotFound)?; // Obtener la cuenta del usuario
            ensure!(account.available_balance >= collateral, Error::<T>::InsufficientCollateral); // Aseguramos que hay suficiente colateral

            // Actualizar el estado del préstamo
            let borrow = Borrow {
                amount, // Monto del préstamo
                collateral, // Colateral proporcionado
                borrow_time: <frame_system::Pallet<T>>::block_number(), // Tiempo en que se tomó el préstamo
            };

            Self::borrows.insert(&who, borrow); // Guardar el préstamo

            // Actualizar el balance disponible
            account.available_balance -= collateral; // Reducir el balance disponible por el colateral
            Self::accounts.insert(&who, account); // Guardar la cuenta actualizada

            Ok(()) // Retornar éxito
        }

        // Función para reembolsar un préstamo
        #[pallet::weight(10_000)]
        pub fn repay(origin: OriginFor<T>, amount: BalanceOf<T>) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?; // Aseguramos que el origen es un usuario firmado

            // Lógica para permitir el reembolso
            let mut borrow = Self::borrows.get(&who).ok_or(Error::<T>::LoanNotFound)?; // Obtener el préstamo
            ensure!(borrow.amount >= amount, Error::<T>::RepayAmountTooHigh); // Aseguramos que el monto a reembolsar no excede el préstamo

            // Actualizar el préstamo y el balance
            borrow.amount -= amount; // Reducir el monto del préstamo
            Self::borrows.insert(&who, borrow); // Guardar el préstamo actualizado

            Ok(()) // Retornar éxito
        }
    }
}