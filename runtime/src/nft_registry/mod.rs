use frame_support::{
    decl_event, decl_module, dispatch::DispatchResult, ensure, weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};

struct NftContract;
type RegistryUid = u64;
type NftUid = u64;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_event!(
    pub enum Event<T> where <T as frame_system::Trait>::Hash {
        DepositAsset(Hash),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as NftRegistry {
        ValidationFn get(validator_of): map RegistryUid => Option<T::AccountId>;
        Nonce: u64;
        RegistryNonce: map RegistryUid => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin  {
        fn deposit_event() = default;

        fn new_registry(origin, validation_fn_addr: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;

            // Generate a uid and check that it's unique
            let nonce = <Nonce>::get();
            let uid = nonce;
            //let uid = (nonce).using_encoded(<T as system::Trait>::Hashing::hash);
            //ensure!(!<ValidationFn<T>>::exists(uid), "This new id for a registry already exists!");

            // Check for overflow on index
            let nplus1 = <Nonce>::get().checked_add(1)
                .ok_or("Nonce overflow when adding a new registry")?;

            // Write state
            <ValidationFn<T>>::insert(&uid, validation_fn_addr);
            <Nonce>::put( nplus1 );

            // Events
            Self::deposit_event(RawEvent::NewRegistry(sender, uid));

            Ok(())
        }

        fn mint(origin,
                uid: RegistryUid,
                parameters: Vec<u8>,            // To be passed into the smart contract
                value: contracts::BalanceOf<T>,  // If currency needs to be passed to contract
                gas_limit: contracts::Gas) -> Result
        {
            // TODO: Needs to ensure signed before anything else
            let sender = ensure_signed(origin)?;

            ensure!(<ValidationFn<T>>::exists(uid), "No registry with this uid exists");

            // Run merkle validation

            // Run custom validation
            let validation_fn = Self::validator_of(uid)
                .ok_or("This should not happen bcs ensure above^")?;

            // Wasm contract should emit an event for success or failure
            <contracts::Module<T>>::call(
                T::Origin::from(RawOrigin::<T::AccountId>::Signed(sender)),
                T::Lookup::unlookup(validation_fn),
                value,
                gas_limit,
                parameters)
        }

        fn finish_mint(origin, uid: RegistryUid) -> Result {
            let sender = ensure_signed(origin)?;

            // Ensure the caller is the validation contract for the corresponding NFT class
            ensure!(Self::validator_of(&uid)
                        .map_or(false, |validator_addr| validator_addr == sender),
                        "Sender must be validator contract for this Nft registry");

            // Assign Nft uid
            let nft_uid = <RegistryNonce>::get(&uid);

            let nplus1 = nft_uid.checked_add(1)
                .ok_or("Overflow when incrementing registry nonce")?;

            // Increment nonce
            <RegistryNonce>::insert(uid, nplus1);

            // Just emit an event
            Self::deposit_event(RawEvent::MintNft(uid, nft_uid));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn validate_proofs() {}
}
