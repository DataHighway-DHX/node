// extern crate env as env;
extern crate roaming_accounting_policies as accounting_policies;
extern crate roaming_agreement_policies as agreement_policies;
extern crate roaming_billing_policies as billing_policies;
extern crate roaming_charging_policies as charging_policies;
extern crate roaming_device_profiles as device_profiles;
extern crate roaming_devices as devices;
extern crate roaming_network_profiles as network_profiles;
extern crate roaming_network_servers as network_servers;
extern crate roaming_networks as networks;
extern crate roaming_operators as operators;
extern crate roaming_organizations as organizations;
extern crate roaming_routing_profiles as routing_profiles;
extern crate roaming_service_profiles as service_profiles;

#[cfg(test)]
mod tests {

    use frame_support::{
        assert_ok,
        parameter_types,
        weights::{
            IdentityFee,
            Weight,
        },
    };

    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{
            BlakeTwo256,
            IdentityLookup,

        },
    };
    pub use pallet_transaction_payment::{
        CurrencyAdapter,
    };
    // Import Config for each runtime module being tested
    use roaming_accounting_policies::{
        Module as RoamingAccountingPolicyModule,
        RoamingAccountingPolicySetting,
        Config as RoamingAccountingPolicyConfig,
    };
    use roaming_agreement_policies::{
        Module as RoamingAgreementPolicyModule,
        RoamingAgreementPolicySetting,
        Config as RoamingAgreementPolicyConfig,
    };
    use roaming_billing_policies::{
        Module as RoamingBillingPolicyModule,
        RoamingBillingPolicySetting,
        Config as RoamingBillingPolicyConfig,
    };
    use roaming_charging_policies::{
        Module as RoamingChargingPolicyModule,
        RoamingChargingPolicySetting,
        Config as RoamingChargingPolicyConfig,
    };
    use roaming_device_profiles::{
        Module as RoamingDeviceProfileModule,
        RoamingDeviceProfileSetting,
        Config as RoamingDeviceProfileConfig,
    };
    use roaming_devices::{
        Module as RoamingDeviceModule,
        Config as RoamingDeviceConfig,
    };
    use roaming_network_profiles::{
        Module as RoamingNetworkProfileModule,
        Config as RoamingNetworkProfileConfig,
    };
    use roaming_network_servers::{
        Module as RoamingNetworkServerModule,
        Config as RoamingNetworkServerConfig,
    };
    use roaming_networks::{
        Module as RoamingNetworkModule,
        Config as RoamingNetworkConfig,
    };
    use roaming_operators::{
        Module as RoamingOperatorModule,
        Config as RoamingOperatorConfig,
    };
    use roaming_organizations::{
        Module as RoamingOrganizationModule,
        Config as RoamingOrganizationConfig,
    };
    use roaming_routing_profiles::{
        Module as RoamingRoutingProfileModule,
        Config as RoamingRoutingProfileConfig,
    };
    use roaming_service_profiles::{
        Module as RoamingServiceProfileModule,
        Config as RoamingServiceProfileConfig,
    };

    // pub fn origin_of(who: &AccountId) -> <Runtime as frame_system::Config>::Origin {
    // 	<Runtime as frame_system::Config>::Origin::signed((*who).clone())
    // }

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
    type Block = frame_system::mocking::MockBlock<Test>;

    frame_support::construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Module, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
            RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
            TransactionPayment: pallet_transaction_payment::{Module, Storage},
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const SS58Prefix: u8 = 33;
    }
    impl frame_system::Config for Test {
        type AccountData = pallet_balances::AccountData<u64>;
        type AccountId = u64;
        type BaseCallFilter = ();
        type BlockHashCount = BlockHashCount;
        type BlockLength = ();
        type BlockNumber = u64;
        type BlockWeights = ();
        type Call = Call;
        type DbWeight = ();
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Header = Header;
        type Index = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type OnKilledAccount = ();
        type OnNewAccount = ();
        type Origin = Origin;
        type PalletInfo = PalletInfo;
        type SS58Prefix = SS58Prefix;
        type SystemWeightInfo = ();
        type Version = ();
    }
    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
    }
    impl pallet_balances::Config for Test {
        type AccountStore = System;
        type Balance = u64;
        type DustRemoval = ();
        type Event = ();
        type ExistentialDeposit = ExistentialDeposit;
        type MaxLocks = ();
        type WeightInfo = ();
    }
    impl pallet_transaction_payment::Config for Test {
        type FeeMultiplierUpdate = ();
        type OnChargeTransaction = CurrencyAdapter<Balances, ()>;
        type TransactionByteFee = ();
        type WeightToFee = IdentityFee<u64>;
    }
    impl RoamingOperatorConfig for Test {
        type Currency = Balances;
        type Event = ();
        type Randomness = RandomnessCollectiveFlip;
        type RoamingOperatorIndex = u64;
    }
    impl RoamingNetworkConfig for Test {
        type Event = ();
        type RoamingNetworkIndex = u64;
    }
    impl RoamingOrganizationConfig for Test {
        type Event = ();
        type RoamingOrganizationIndex = u64;
    }
    impl RoamingNetworkServerConfig for Test {
        type Event = ();
        type RoamingNetworkServerIndex = u64;
    }
    impl RoamingAgreementPolicyConfig for Test {
        type Event = ();
        type RoamingAgreementPolicyActivationType = Vec<u8>;
        type RoamingAgreementPolicyIndex = u64;
    }
    impl RoamingAccountingPolicyConfig for Test {
        type Event = ();
        type RoamingAccountingPolicyDownlinkFeeFactor = u32;
        type RoamingAccountingPolicyIndex = u64;
        type RoamingAccountingPolicyType = Vec<u8>;
        type RoamingAccountingPolicyUplinkFeeFactor = u32;
    }
    impl RoamingRoutingProfileConfig for Test {
        type Event = ();
        type RoamingRoutingProfileAppServer = Vec<u8>;
        type RoamingRoutingProfileIndex = u64;
    }
    impl RoamingDeviceConfig for Test {
        type Event = ();
        type RoamingDeviceIndex = u64;
    }
    impl RoamingServiceProfileConfig for Test {
        type Event = ();
        type RoamingServiceProfileDownlinkRate = u32;
        type RoamingServiceProfileIndex = u64;
        type RoamingServiceProfileUplinkRate = u32;
    }
    impl RoamingBillingPolicyConfig for Test {
        type Event = ();
        type RoamingBillingPolicyIndex = u64;
    }
    impl RoamingChargingPolicyConfig for Test {
        type Event = ();
        type RoamingChargingPolicyIndex = u64;
    }
    impl RoamingNetworkProfileConfig for Test {
        type Event = ();
        type RoamingNetworkProfileIndex = u64;
    }
    impl RoamingDeviceProfileConfig for Test {
        type Event = ();
        type RoamingDeviceProfileDevAddr = Vec<u8>;
        type RoamingDeviceProfileDevEUI = Vec<u8>;
        type RoamingDeviceProfileIndex = u64;
        type RoamingDeviceProfileJoinEUI = Vec<u8>;
        type RoamingDeviceProfileVendorID = Vec<u8>;
    }

    pub type RoamingOperatorTestModule = RoamingOperatorModule<Test>;
    pub type RoamingNetworkTestModule = RoamingNetworkModule<Test>;
    pub type RoamingOrganizationTestModule = RoamingOrganizationModule<Test>;
    pub type RoamingNetworkServerTestModule = RoamingNetworkServerModule<Test>;
    pub type RoamingAgreementPolicyTestModule = RoamingAgreementPolicyModule<Test>;
    pub type RoamingAccountingPolicyTestModule = RoamingAccountingPolicyModule<Test>;
    pub type RoamingRoutingProfileTestModule = RoamingRoutingProfileModule<Test>;
    pub type RoamingDeviceTestModule = RoamingDeviceModule<Test>;
    pub type RoamingServiceProfileTestModule = RoamingServiceProfileModule<Test>;
    pub type RoamingBillingPolicyTestModule = RoamingBillingPolicyModule<Test>;
    pub type RoamingChargingPolicyTestModule = RoamingChargingPolicyModule<Test>;
    pub type RoamingNetworkProfileTestModule = RoamingNetworkProfileModule<Test>;
    pub type RoamingDeviceProfileTestModule = RoamingDeviceProfileModule<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30)],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    // Create Users on Data Highway
    #[test]
    fn setup_users() {
        new_test_ext().execute_with(|| {
            assert_eq!(Balances::free_balance(1), 10);
            assert_eq!(Balances::free_balance(2), 20);
            assert_eq!(Balances::free_balance(3), 30);
            assert_eq!(Balances::reserved_balance(&1), 0);
            // FIXME - why can't I query `total_balance` from the Balances frame
            // assert_eq!(Balances::total_balance(&1), 0);
        });
    }

    #[test]
    fn integration_test() {
        new_test_ext().execute_with(|| {
            // env::config::set_test_env();

            // Create Network Operators
            //
            // Create two network operators with two different admin users

            // Call Functions
            assert_ok!(RoamingOperatorTestModule::create(Origin::signed(0))); // MXC
            assert_ok!(RoamingOperatorTestModule::create(Origin::signed(1))); // TEX
            // FIXME - create a User runtime module that may be an admin, and
            // may be assigned to an Operator and an Organization

            // Verify Storage
            assert_eq!(RoamingOperatorTestModule::roaming_operators_count(), 2);
            assert!(RoamingOperatorTestModule::roaming_operator(0).is_some()); // MXC
            assert!(RoamingOperatorTestModule::roaming_operator(1).is_some()); // TEX
            assert_eq!(RoamingOperatorTestModule::roaming_operator_owner(0), Some(0));
            assert_eq!(RoamingOperatorTestModule::roaming_operator_price(0), None);

            // Create Networks
            //
            // Create two networks with the same admin user.
            // Assign them to the same network operator, by the owner of that network operator

            // Call Functions
            assert!(RoamingNetworkTestModule::exists_roaming_network(0).is_err(), "Roaming network does not exist yet");
            assert!(
                RoamingNetworkTestModule::is_roaming_network_owner(0, 1).is_err(),
                "Roaming network does not exist yet"
            );
            assert_ok!(RoamingNetworkTestModule::create(Origin::signed(0)));
            assert_ok!(RoamingNetworkTestModule::create(Origin::signed(1)));
            assert!(RoamingNetworkTestModule::exists_roaming_network(0).is_ok());
            assert!(RoamingNetworkTestModule::exists_roaming_network(1).is_ok());
            assert_ok!(RoamingNetworkTestModule::is_roaming_network_owner(0, 0));
            assert_ok!(RoamingNetworkTestModule::is_roaming_network_owner(1, 1));
            // Assign each network to the MXC network operator, by the owner/creator of the MXC network operator
            assert_ok!(RoamingNetworkTestModule::assign_network_to_operator(Origin::signed(0), 0, 0));
            assert_ok!(RoamingNetworkTestModule::assign_network_to_operator(Origin::signed(0), 1, 0));

            // Verify Storage
            assert_eq!(RoamingNetworkTestModule::roaming_networks_count(), 2);
            assert!(RoamingNetworkTestModule::roaming_network(0).is_some());
            assert!(RoamingNetworkTestModule::roaming_network(1).is_some());
            assert_eq!(RoamingNetworkTestModule::roaming_network_owner(0), Some(0));
            assert_eq!(RoamingNetworkTestModule::roaming_network_price(0), None);

            // Create Organizations
            //
            // Create three organizations with the same admin user.

            // Call Functions
            assert_ok!(RoamingOrganizationTestModule::create(Origin::signed(0)));
            assert_ok!(RoamingOrganizationTestModule::create(Origin::signed(1)));
            assert_ok!(RoamingOrganizationTestModule::create(Origin::signed(2)));
            // FIXME - assign Users to the organizations

            // Verify Storage
            assert_eq!(RoamingOrganizationTestModule::roaming_organizations_count(), 3);

            // Create Network Servers
            //
            // Create a network server with the same admin user.

            // Call Functions
            assert_ok!(RoamingNetworkServerTestModule::create(Origin::signed(0)));
            assert_ok!(RoamingNetworkServerTestModule::create(Origin::signed(1)));

            // Verify Storage
            assert_eq!(RoamingNetworkServerTestModule::roaming_network_servers_count(), 2);

            // Create Roaming Accounting Policy

            // Call Functions
            assert_ok!(RoamingAccountingPolicyTestModule::create(Origin::signed(0)));
            // Note: This step is optional
            assert_ok!(RoamingAccountingPolicyTestModule::assign_accounting_policy_to_network(Origin::signed(0), 0, 0));
            assert_eq!(RoamingAccountingPolicyTestModule::roaming_accounting_policy_owner(0), Some(0));
            assert_ok!(RoamingAccountingPolicyTestModule::set_config(
                Origin::signed(0),
                0,                              // accounting_policy_id
                Some(b"subscription".to_vec()), // policy_type
                Some(200),                      // subscription_fee
                Some(15),                       // uplink_fee_factor
                Some(10),                       // downlink_fee_factor
            ));

            // Verify Storage
            assert_eq!(RoamingAccountingPolicyTestModule::roaming_accounting_policies_count(), 1);
            assert_eq!(
                RoamingAccountingPolicyTestModule::roaming_accounting_policy_settings(0),
                Some(RoamingAccountingPolicySetting {
                    policy_type: b"subscription".to_vec(), // policy_type
                    subscription_fee: 200,                 // subscription_fee
                    uplink_fee_factor: 15,                 // uplink_fee_factor
                    downlink_fee_factor: 10,               // downlink_fee_factor
                })
            );

            // Create Roaming Agreement Policy

            // Call Functions
            assert_ok!(RoamingAgreementPolicyTestModule::create(Origin::signed(0)));
            // Note: This step is optional since it will be assigned to a network when
            // a associated with a network (roaming base) profile
            assert_ok!(RoamingAgreementPolicyTestModule::assign_agreement_policy_to_network(Origin::signed(0), 0, 0));
            // assert_eq!(
            //     RoamingAgreementPolicyTestModule::exists_roaming_agreement_policy(0),
            //     Ok(RoamingAgreementPolicy([0; 16]))
            // );
            assert_eq!(RoamingAgreementPolicyTestModule::roaming_agreement_policy_owner(0), Some(0));
            assert_ok!(RoamingAgreementPolicyTestModule::set_config(
                Origin::signed(0),
                0,
                Some(b"passive".to_vec()),
                Some(2019)
            ));

            // Verify Storage
            assert_eq!(RoamingAgreementPolicyTestModule::roaming_agreement_policies_count(), 1);
            assert_eq!(
                RoamingAgreementPolicyTestModule::roaming_agreement_policy_settings(0),
                Some(RoamingAgreementPolicySetting {
                    policy_activation_type: b"passive".to_vec(),
                    policy_expiry_block: 2019,
                })
            );

            // Create Roaming Routing Profile

            // Call Functions
            assert_ok!(RoamingRoutingProfileTestModule::create(Origin::signed(0)));
            assert_eq!(RoamingRoutingProfileTestModule::roaming_routing_profile_owner(0), Some(0));
            assert_ok!(RoamingRoutingProfileTestModule::set_app_server(
                Origin::signed(0),
                0,                          // routing_profile_id
                Some(b"10.0.0.1".to_vec()), // app server
            ));

            // Verify Storage
            assert_eq!(RoamingRoutingProfileTestModule::roaming_routing_profiles_count(), 1);
            assert_eq!(
                RoamingRoutingProfileTestModule::roaming_routing_profile_app_server(0),
                Some(b"10.0.0.1".to_vec())
            );

            // Create Service Profile

            // Call Functions
            assert_ok!(RoamingServiceProfileTestModule::create(Origin::signed(0)));
            assert_eq!(RoamingServiceProfileTestModule::roaming_service_profile_owner(0), Some(0));
            // Note: Optional since it will be assigned to a network when
            // a associated with a network (roaming base) profile, but we can override it to apply to specific
            // network server this way.
            assert_ok!(RoamingServiceProfileTestModule::assign_service_profile_to_network_server(
                Origin::signed(0),
                0,
                0
            ));
            assert_ok!(RoamingServiceProfileTestModule::set_uplink_rate(
                Origin::signed(0),
                0,        // service_profile_id
                Some(10), // uplink_rate
            ));
            assert_ok!(RoamingServiceProfileTestModule::set_downlink_rate(
                Origin::signed(0),
                0,       // service_profile_id
                Some(5), // downlink_rate
            ));

            // Verify Storage
            assert_eq!(RoamingServiceProfileTestModule::roaming_service_profiles_count(), 1);
            assert_eq!(RoamingServiceProfileTestModule::roaming_service_profile_uplink_rate(0), Some(10));
            assert_eq!(RoamingServiceProfileTestModule::roaming_service_profile_downlink_rate(0), Some(5));

            // Create Billing Policy

            // Call Functions
            assert_ok!(RoamingBillingPolicyTestModule::create(Origin::signed(0)));
            // Note: This step is optional since it will be assigned to a network and operator when
            // associated with a network (roaming base) profile
            // assert_ok!(
            //     RoamingBillingPolicyTestModule::assign_billing_policy_to_operator(
            //         Origin::signed(0),
            //         0,
            //         0
            //     )
            // );
            // Note: This step is optional since it will be assigned to a network and operator when
            // associated with a network (roaming base) profile
            // assert_ok!(
            //     RoamingBillingPolicyTestModule::assign_billing_policy_to_network(
            //         Origin::signed(0),
            //         0,
            //         0
            //     )
            // );
            assert_eq!(RoamingBillingPolicyTestModule::roaming_billing_policy_owner(0), Some(0));
            assert_ok!(RoamingBillingPolicyTestModule::set_config(
                Origin::signed(0),
                0,
                Some(102020), // next_billing_at
                Some(30)      // frequency_in_days
            ));

            // Verify Storage
            assert_eq!(RoamingBillingPolicyTestModule::roaming_billing_policies_count(), 1);
            assert_eq!(
                RoamingBillingPolicyTestModule::roaming_billing_policy_settings(0),
                Some(RoamingBillingPolicySetting {
                    policy_next_billing_at_block: 102020,
                    policy_frequency_in_blocks: 30,
                })
            );

            // Create Charging Policy

            // Call Functions
            assert_ok!(RoamingChargingPolicyTestModule::create(Origin::signed(0)));
            // Note: This step is optional since it will be assigned to a network and operator when
            // associated with a network (roaming base) profile
            // assert_ok!(
            //     RoamingChargingPolicyTestModule::assign_charging_policy_to_operator(
            //         Origin::signed(0),
            //         0,
            //         0
            //     )
            // );
            // Note: This step is optional since it will be assigned to a network and operator when
            // associated with a network (roaming base) profile
            // assert_ok!(
            //     RoamingChargingPolicyTestModule::assign_charging_policy_to_network(
            //         Origin::signed(0),
            //         0,
            //         0
            //     )
            // );
            assert_eq!(RoamingChargingPolicyTestModule::roaming_charging_policy_owner(0), Some(0));
            assert_ok!(RoamingChargingPolicyTestModule::set_config(
                Origin::signed(0),
                0,
                Some(102020), // next_charging_at
                Some(7)       // frequency_in_days
            ));

            // Verify Storage
            assert_eq!(RoamingChargingPolicyTestModule::roaming_charging_policies_count(), 1);
            assert_eq!(
                RoamingChargingPolicyTestModule::roaming_charging_policy_settings(0),
                Some(RoamingChargingPolicySetting {
                    policy_next_charging_at_block: 102020,
                    policy_delay_after_billing_in_blocks: 7,
                })
            );

            // TODO - add Dispute Policy

            // TODO - add Adjustment Policy

            // Create Network Profiles

            // Call Functions
            assert_ok!(RoamingNetworkProfileTestModule::create(Origin::signed(0)));
            assert_ok!(RoamingNetworkProfileTestModule::create(Origin::signed(0)));
            assert_ok!(RoamingNetworkProfileTestModule::create(Origin::signed(1)));
            assert_eq!(RoamingNetworkProfileTestModule::roaming_network_profile_owner(0), Some(0));
            assert_eq!(RoamingNetworkProfileTestModule::roaming_network_profile_owner(1), Some(0));
            assert_eq!(RoamingNetworkProfileTestModule::roaming_network_profile_owner(2), Some(1));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_network(Origin::signed(0), 0, 0));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_operator(Origin::signed(0), 0, 0));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_network(Origin::signed(0), 1, 0));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_operator(Origin::signed(0), 1, 0));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_network(Origin::signed(1), 2, 1));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_operator(Origin::signed(0), 2, 0));
            assert_ok!(RoamingNetworkProfileTestModule::set_device_access_allowed(Origin::signed(0), 0, true));
            assert_ok!(RoamingNetworkProfileTestModule::set_device_access_allowed(Origin::signed(0), 1, true));
            assert_ok!(RoamingNetworkProfileTestModule::set_device_access_allowed(Origin::signed(1), 2, false));
            // If we know the whitelisted network then we know the whitelisted operator too
            // Network Profile 0 - Whitelist MXC
            assert_ok!(RoamingNetworkProfileTestModule::add_whitelisted_network(
                Origin::signed(0),
                0, // network_profile_id
                0, // network_id
            ));
            // assert_ok!(
            //     RoamingNetworkProfileTestModule::add_whitelisted_network(
            //         Origin::signed(0),
            //         0, // network_profile_id
            //         0, // operator_id
            //     )
            // );
            // Network Profile 2 - Whitelist TEX (Any of its networks)
            // assert_ok!(
            //     RoamingNetworkProfileTestModule::add_whitelisted_operator(
            //         Origin::signed(0),
            //         1, // network_profile_id
            //         1, // operator_id
            //         // FIXME - add all the policies and profiles that will be associated with this whitelisting
            //     )
            // );

            // Verify Storage
            assert_eq!(RoamingNetworkProfileTestModule::roaming_network_profiles_count(), 3);
            assert_eq!(
                RoamingNetworkProfileTestModule::roaming_network_profile_whitelisted_networks(0),
                Some([0].to_vec())
            );
            // TODO - validate whitelisted operators

            // FIXME - we need to rethink storage of whitelisted networks and operator, storing together would
            // work better since network id may not be unique across different operators.

            // Create Device

            // Call Functions
            assert_ok!(RoamingDeviceTestModule::create(Origin::signed(0)));
            assert_eq!(RoamingDeviceTestModule::roaming_device_owner(0), Some(0));
            assert_ok!(RoamingDeviceTestModule::assign_device_to_organization(Origin::signed(2), 0, 2));
            assert_ok!(RoamingDeviceTestModule::assign_device_to_network_server(Origin::signed(1), 0, 1));

            // Verify Storage
            assert_eq!(RoamingDeviceTestModule::roaming_devices_count(), 1);

            // Create Device Profile

            // Call Functions
            assert_ok!(RoamingDeviceProfileTestModule::create(Origin::signed(0)));
            assert_ok!(RoamingDeviceProfileTestModule::assign_device_profile_to_device(Origin::signed(0), 0, 0));
            assert_eq!(RoamingDeviceProfileTestModule::roaming_device_profile_owner(0), Some(0));
            assert_ok!(RoamingDeviceProfileTestModule::set_config(
                Origin::signed(0),
                0,
                Some(b"1234".to_vec()), // device_profile_devaddr
                Some(b"5678".to_vec()), // device_profile_deveui
                Some(b"6789".to_vec()), // device_profile_joineui
                Some(b"1000".to_vec()), // device_profile_vendorid
            ));

            // Verify Storage
            assert_eq!(RoamingDeviceProfileTestModule::roaming_device_profiles_count(), 1);
            assert_eq!(
                RoamingDeviceProfileTestModule::roaming_device_profile_settings(0),
                Some(RoamingDeviceProfileSetting {
                    device_profile_devaddr: b"1234".to_vec(),
                    device_profile_deveui: b"5678".to_vec(),
                    device_profile_joineui: b"6789".to_vec(),
                    device_profile_vendorid: b"1000".to_vec(),
                })
            );

            // MMD-1022 - Spec for how M2 Pro will interact on blockchain

            // Set an account balance to each account for transaction fees
            assert_ok!(Balances::set_balance(Origin::root(), 1, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 2, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 3, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 4, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 5, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 6, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 7, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 8, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 9, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 10, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 11, 1_000, 0));
            assert_ok!(Balances::set_balance(Origin::root(), 12, 1_000, 0));

            // Identity for accounts and their hardware devices

            let sudo = Origin::root();
            let mxc_account_0_id = 0;
            let mxc_account_signed = Origin::signed(mxc_account_0_id);

            let user_account_1_id = 1;
            let user_account_1_signed = Origin::signed(user_account_1_id);
            // Identity info of account_id 1
            let user_account_1_info = Some(IdentityInfo {
                email: b"luke@mxc.org".to_vec()
            });
            assert_ok!(Identity::set_identity(registrar, user_account_1_info)); // Set by registrar

            let user_account_2_id = 12;
            let user_account_2_signed = Origin::signed(user_account_2_id);
            // Identity info of account_id 2
            let user_account_2_info = Some(IdentityInfo {
                email: b"test@mxc.org".to_vec()
            });
            assert_ok!(Identity::set_identity(registrar, user_account_2_info)); // Set by registrar
            // assert_ok!(Identity::add_sub(user_account_1_signed, user_account_2_id, user_account_2_info);

            // Registrar needs to already exist before `register_gateway` or `register_device` is called
            let registrar_account_id = 11;
            let registrar = Origin::signed(registrar_account_id);
            let registrar_index = 0u64; // Registrar Index

            assert_eq!(Identity::add_registrar(sudo.clone(), registrar.clone()), registrar_index); // Sudo required
            let registrar_judgement_fee = 100u32;
            assert_ok!(Identity::set_fee(sudo.clone(), registrar_index, registrar_judgement_fee.clone())); // Sudo required
            assert_ok!(Identity::set_fields(sudo.clone(), registrar_index, registrar.clone(), IdentityFields::Email)); // Sudo required
            let max_fee = 1u32;

            // TODO - the below `create` needs to create an account id for the object (i.e. operator_account_id)
            assert_ok!(RoamingOperatorTestModule::create(mxc_account_signed));
            let mxc_roaming_operator_id = 2; // Hard-coded. Generated by above line
            // TODO - the below `create` needs to create an account id for the object (i.e. network_account_id)
            assert_ok!(RoamingNetworkTestModule::create(mxc_account_signed));
            let mxc_roaming_network_id = 3; // Hard-coded. Generated by above line
            assert_ok!(RoamingNetworkTestModule::is_roaming_network_owner(mxc_roaming_network_id, mxc_account_0_id));
            // Assign each network to the MXC network operator, by the owner/creator of the MXC network operator
            assert_ok!(RoamingNetworkTestModule::assign_network_to_operator(mxc_account_signed, mxc_roaming_network_id, mxc_roaming_operator_id));
            assert_eq!(RoamingNetworkTestModule::roaming_network_owner(mxc_account_0_id), Some(mxc_roaming_network_id));
            // TODO - the below `create` needs to create an account id for the object (i.e. network_server_account_id)
            assert_ok!(RoamingNetworkServerTestModule::create(mxc_account_signed)); // Supernode
            let mxc_network_server_id = 4; // Hard-coded. Generated by above line

            // Call Functions
            assert_ok!(RoamingNetworkProfileTestModule::create(mxc_account_signed));
            let mxc_roaming_network_profile_id = 5; // Hard-coded. Generated by above line
            assert_eq!(RoamingNetworkProfileTestModule::roaming_network_profile_owner(mxc_roaming_network_profile_id), Some(mxc_roaming_network_id));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_network(mxc_account_signed, mxc_roaming_network_profile_id, mxc_roaming_network_id));
            assert_ok!(RoamingNetworkProfileTestModule::assign_network_profile_to_operator(mxc_account_signed, mxc_roaming_network_profile_id, mxc_roaming_operator_id));
            assert_ok!(RoamingNetworkProfileTestModule::set_device_access_allowed(mxc_account_signed, mxc_roaming_network_profile_id, true));
            // Network Profile 0 - Whitelist MXC
            assert_ok!(RoamingNetworkProfileTestModule::add_whitelisted_network(
                mxc_account_signed,
                mxc_roaming_network_profile_id, // network_profile_id
                mxc_roaming_network_id, // network_id
            ));

            // Identity is assigned to user wallet upon gateway registration
            // ** Gateway calls this extrinsic function **
            assert_ok!(RoamingGatewayTestModule::register_gateway(user_account_1_signed));
                // ** Important note: Behind the scenes, the above function should do all the following internally
                // so we need to provide `register_gateway` with sufficient arguments
                // TODO - the below `create` needs to create an account id for the object (i.e. user_organization_account_id)
                assert_ok!(RoamingOrganizationTestModule::create(user_account_1_signed));
                let user_organization_id = 6; // Hard-coded. Generated by above line
                // TODO - add this extrinsic function `assign_user_to_organization`
                assert_ok!(RoamingGatewayTestModule::assign_user_to_organization(user_account_1_signed, user_account_1_id, user_organization_id));
                // TODO - add this roaming gateway pallet
                // Create Gateway
                // TODO - the below `create` needs to create an account id for the object (i.e. gateway_account_id)
                assert_ok!(RoamingGatewayTestModule::create(user_account_1_signed));
                // M2 Pro
                let gateway_account_id = 7; // Hard-coded. Generated by above line
                let gateway_account_signed = Origin::signed(gateway_account_id);
                assert_eq!(RoamingGatewayTestModule::roaming_gateway_owner(gateway_account_id), Some(7));
                assert_ok!(RoamingGatewayTestModule::assign_gateway_to_organization(user_account_1_signed, gateway_account_id, user_organization_id));
                assert_ok!(RoamingGatewayTestModule::assign_gateway_to_network_server(mxc_account_signed, gateway_account_id, mxc_network_server_id));
                assert_ok!(RoamingGatewayProfileTestModule::create(user_account_1_signed));
                let gateway_profile_id = 8; // Hard-coded. Generated by above line
                assert_ok!(RoamingGatewayProfileTestModule::assign_gateway_profile_to_gateway(user_account_1_signed, gateway_profile_id, gateway_account_id));
                assert_ok!(RoamingGatewayProfileTestModule::set_config(
                    user_account_1_signed,
                    gateway_profile_id,
                    Some(b"1234".to_vec()), // gateway_profile_mac
                    Some(b"1000".to_vec()), // gateway_profile_vendorid
                ));
                let gateway_profile_settings_0_id = 0; // Hard-coded. Generated by above line

                // Identity info of hardware gateway (gateway)
                let fetched_gateway_profile_setting = <RoamingGatewayProfileSettings<T>>::get(gateway_profile_settings_0_id);
                // https://dev.datahighway.com/docs/en/whitepaper#gateway-setup--staking
                let gateway_account_roaming_data = (
                    supernode_home_account_id: <RoamingGatewayNetworkServers<T>>::get(0),
                    gateway_id_mac: fetched_gateway_profile_setting.gateway_profile_mac.unwrap(),
                    gateway_profile_vendorid: fetched_gateway_profile_setting.gateway_profile_vendorid.unwrap(),
                ); // Tuple
                // TODO - how do we store this using Identity pallet, as it says that values greater than u32
                // such as account_id's that are u64 will be truncated but we want to associate the account_id
                // https://github.com/paritytech/substrate/blob/master/frame/identity/src/lib.rs
                let gateway_account_additional: Vec<HardwareData<u64, (u64, u64)>> = Vec::new();
                gateway_account_additional.push(gateway_account_id, gateway_account_roaming_data);
                // Reference: https://github.com/paritytech/substrate/blob/master/frame/identity/src/lib.rs#L255
                let gateway_account_info = Some(IdentityInfo {
                    additional: gateway_account_additional
                });
                assert_ok!(Identity::set_identity(registrar, gateway_account_info)); // Set by registrar
                // Identity is assigned to user wallet upon device registration
                assert_ok!(Identity::add_sub(user_account_1_signed, gateway_account_id, gateway_account_info);
                // Request judgement of user wallet identity by the registrar
                // to confirm that their sub-identities (end device and gateway) do infact belong to the user
                assert_ok!(Identity::request_judgement(user_account_1_signed, registrar_index, max_fee));
                assert_ok!(Identity::provide_judgement(registrar, registrar_index, user_account_1_signed, registrar_judgement_fee.clone()));

            // Identity is assigned to user wallet upon device registration
            // ** Device calls this extrinsic function **
            assert_ok!(RoamingDeviceTestModule::register_device(user_account_1_signed));
                // Create End Device
                // TODO - the below `create` needs to create an account id for the object (i.e. device_account_id)
                assert_ok!(RoamingDeviceTestModule::create(user_account_1_signed));
                let device_account_id = 9; // Hard-coded. Generated by above line
                let device_account_signed = Origin::signed(device_account_id);
                assert_eq!(RoamingDeviceTestModule::roaming_device_owner(device_account_id), Some(9));
                assert_ok!(RoamingDeviceTestModule::assign_device_to_organization(user_account_1_signed, device_account_id, user_organization_id));
                assert_ok!(RoamingDeviceTestModule::assign_device_to_network_server(mxc_account_signed, device_account_id, mxc_network_server_id));
                // Note: We could create the RoamingDeviceProfile using the signed RoamingDevice account instead of the user
                // to simplify the associations, but for now we will leave it this way for now and call `assign_..` to make the assocations instead.
                assert_ok!(RoamingDeviceProfileTestModule::create(user_account_1_signed));
                let device_profile_id = 10; // Hard-coded. Generated by above line
                assert_ok!(RoamingDeviceProfileTestModule::assign_device_profile_to_device(user_account_1_signed, device_profile_id, device_account_id));
                assert_ok!(RoamingDeviceProfileTestModule::set_config(
                    user_account_1_signed,
                    device_profile_id,
                    Some(b"1234".to_vec()), // device_profile_devaddr
                    Some(b"5678".to_vec()), // device_profile_deveui
                    Some(b"6789".to_vec()), // device_profile_joineui
                    Some(b"1000".to_vec()), // device_profile_vendorid
                ));
                let device_profile_settings_0_id = 0; // Hard-coded. Generated by above line

                // Identity info of hardware device (end device)
                let fetched_device_profile_setting = <RoamingDeviceProfileSettings<T>>::get(device_profile_settings_0_id);
                let device_account_roaming_data = (
                    roaming_device_index: fetched_device_profile_setting.roaming_device_index,
                    roaming_device_profile_index: fetched_device_profile_setting.roaming_device_profile_index,
                    roaming_device_profile_devaddr: fetched_device_profile_setting.device_profile_devaddr,
                    roaming_device_profile_vendorid: fetched_device_profile_setting.device_profile_vendorid,
                ); // Tuple
                // TODO - how do we store this using Identity pallet, as it says that values greater than u32
                // such as account_id's that are u64 will be truncated but we want to associate the account_id
                // https://github.com/paritytech/substrate/blob/master/frame/identity/src/lib.rs
                let device_account_additional: Vec<HardwareData<u64, (u64, u64)>> = Vec::new();
                device_account_additional.push(device_account_signed, device_account_roaming_data);
                // Reference: https://github.com/paritytech/substrate/blob/master/frame/identity/src/lib.rs#L255
                let device_account_info = Some(IdentityInfo {
                    additional: device_account_additional
                });
                assert_ok!(Identity::set_identity(registrar, device_account_info)); // Set by registrar

                // Identity is assigned to user wallet upon gateway registration
                assert_ok!(Identity::add_sub(user_account_1_signed, device_account_id, device_account_info);
                // Request judgement of user wallet identity by the registrar
                // to confirm that their sub-identities (end device and gateway) do infact belong to the user
                assert_ok!(Identity::request_judgement(user_account_1_signed, registrar_index, max_fee));
                assert_ok!(Identity::provide_judgement(registrar, registrar_index, user_account_1_signed, registrar_judgement_fee.clone()));


            // Transfer identity ownership of M2 Pro Gateway to a different user
            // ** Gateway calls this extrinsic function **
            // TODO - the below `transfer_gateway_owner` extrinsic needs to be added
            assert_ok!(RoamingGatewayTestModule::transfer_gateway_owner(user_account_1_signed, gateway_account_id, user_account_2_id));
                assert_eq!(RoamingGatewayTestModule::roaming_gateway_owner(gateway_account_id), Some(user_account_2_id));
                // Note: If a user sells hardware, we need them to remove the hardware from being one of their sub-identities
                assert_ok!(Identity::remove_sub(user_account_1_signed, gateway_account_id);
                assert_ok!(Identity::add_sub(user_account_2_id, gateway_account_id, gateway_account_info);

            // Transfer identity ownership of end device to a different user
            // ** Device calls this extrinsic function **
            // TODO - the below `transfer_device_owner` extrinsic needs to be added
            assert_ok!(RoamingDeviceTestModule::transfer_device_owner(user_account_1_signed, device_account_id, user_account_2_id));
                assert_eq!(RoamingDeviceTestModule::roaming_device_owner(device_account_id), Some(user_account_2_id));
                // Note: If a user sells hardware, we need them to remove the hardware from being one of their sub-identities
                assert_ok!(Identity::remove_sub(user_account_1_signed, device_account_id);
                assert_ok!(Identity::add_sub(user_account_2_id, device_account_id, device_account_info);

            // TODO - use Proxy to allow another user to make calls on behalf of an identity
            // assert_ok!(Proxy::add_proxy(sudo.clone(), 3, ProxyType::Any, 1));
        });
    }
}
