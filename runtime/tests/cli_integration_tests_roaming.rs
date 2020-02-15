// extern crate env as env;
extern crate roaming_operators as operators;
extern crate roaming_networks as networks;
extern crate roaming_organizations as organizations;
extern crate roaming_network_servers as network_servers;
extern crate roaming_agreement_policies as agreement_policies;
extern crate roaming_accounting_policies as accounting_policies;
extern crate roaming_routing_profiles as routing_profiles;
extern crate roaming_devices as devices;
extern crate roaming_service_profiles as service_profiles;
extern crate roaming_billing_policies as billing_policies;
extern crate roaming_charging_policies as charging_policies;
extern crate roaming_network_profiles as network_profiles;
extern crate roaming_device_profiles as device_profiles;

#[cfg(test)]
mod tests {
    use super::*;

	use sp_core::H256;
	use frame_support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};
    // Import Trait for each runtime module being tested
    use roaming_operators::{
        Module as RoamingOperatorModule,
        Trait as RoamingOperatorTrait,
    };
    use roaming_networks::{
        Module as RoamingNetworkModule,
        Trait as RoamingNetworkTrait,
    };
    use roaming_organizations::{
        Module as RoamingOrganizationModule,
        Trait as RoamingOrganizationTrait,
    };
    use roaming_network_servers::{
        Module as RoamingNetworkServerModule,
        Trait as RoamingNetworkServerTrait,
    };
    use roaming_agreement_policies::{
        Module as RoamingAgreementPolicyModule,
        RoamingAgreementPolicy,
        RoamingAgreementPolicyConfig,
        Trait as RoamingAgreementPolicyTrait,
    };
    use roaming_accounting_policies::{
        Module as RoamingAccountingPolicyModule,
        RoamingAccountingPolicy,
        RoamingAccountingPolicyConfig,
        Trait as RoamingAccountingPolicyTrait,
    };
    use roaming_routing_profiles::{
        Module as RoamingRoutingProfileModule,
        RoamingRoutingProfile,
        Trait as RoamingRoutingProfileTrait,
    };
    use roaming_devices::{
        Module as RoamingDeviceModule,
        RoamingDevice,
        Trait as RoamingDeviceTrait,
    };
    use roaming_service_profiles::{
        Module as RoamingServiceProfileModule,
        RoamingServiceProfile,
        Trait as RoamingServiceProfileTrait,
    };
    use roaming_billing_policies::{
        Module as RoamingBillingPolicyModule,
        RoamingBillingPolicy,
        RoamingBillingPolicyConfig,
        Trait as RoamingBillingPolicyTrait,
    };
    use roaming_charging_policies::{
        Module as RoamingChargingPolicyModule,
        RoamingChargingPolicy,
        RoamingChargingPolicyConfig,
        Trait as RoamingChargingPolicyTrait,
    };
    use roaming_network_profiles::{
        Module as RoamingNetworkProfileModule,
        RoamingNetworkProfile,
        Trait as RoamingNetworkProfileTrait,
    };
    use roaming_device_profiles::{
        Module as RoamingDeviceProfileModule,
        RoamingDeviceProfile,
        RoamingDeviceProfileConfig,
        Trait as RoamingDeviceProfileTrait,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        // type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type CreationFee = ();
    }
    impl transaction_payment::Trait for Test {
        type Currency = Balances;
        type OnTransactionPayment = ();
        type TransactionBaseFee = ();
        type TransactionByteFee = ();
        type WeightToFee = ();
        type FeeMultiplierUpdate = ();
    }
    impl RoamingOperatorTrait for Test {
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
        type RoamingOperatorIndex = u64;
    }
    impl RoamingNetworkTrait for Test {
        type Event = ();
        type RoamingNetworkIndex = u64;
    }
    impl RoamingOrganizationTrait for Test {
        type Event = ();
        type RoamingOrganizationIndex = u64;
    }
    impl RoamingNetworkServerTrait for Test {
        type Event = ();
        type RoamingNetworkServerIndex = u64;
    }
    impl RoamingAgreementPolicyTrait for Test {
        type Event = ();
        type RoamingAgreementPolicyIndex = u64;
        type RoamingAgreementPolicyActivationType = Vec<u8>;
        type RoamingAgreementPolicyExpiry = u64;
    }
    impl RoamingAccountingPolicyTrait for Test {
        type Event = ();
        type RoamingAccountingPolicyIndex = u64;
        type RoamingAccountingPolicyType = Vec<u8>;
        type RoamingAccountingPolicyUplinkFeeFactor = u32;
        type RoamingAccountingPolicyDownlinkFeeFactor = u32;
    }
    impl RoamingRoutingProfileTrait for Test {
        type Event = ();
        type RoamingRoutingProfileIndex = u64;
        type RoamingRoutingProfileAppServer = Vec<u8>;
    }
    impl RoamingDeviceTrait for Test {
        type Event = ();
        type RoamingDeviceIndex = u64;
    }
    impl RoamingServiceProfileTrait for Test {
        type Event = ();
        type RoamingServiceProfileIndex = u64;
        type RoamingServiceProfileUplinkRate = u32;
        type RoamingServiceProfileDownlinkRate = u32;
    }
    impl RoamingBillingPolicyTrait for Test {
        type Event = ();
        type RoamingBillingPolicyIndex = u64;
        type RoamingBillingPolicyNextBillingAt = u64;
        type RoamingBillingPolicyFrequencyInDays = u64;
    }
    impl RoamingChargingPolicyTrait for Test {
        type Event = ();
        type RoamingChargingPolicyIndex = u64;
        type RoamingChargingPolicyNextChargingAt = u64;
        type RoamingChargingPolicyDelayAfterBillingInDays = u64;
    }
    impl RoamingNetworkProfileTrait for Test {
        type Event = ();
        type RoamingNetworkProfileIndex = u64;
    }
    impl RoamingDeviceProfileTrait for Test {
        type Event = ();
        type RoamingDeviceProfileIndex = u64;
        type RoamingDeviceProfileDevAddr = Vec<u8>;
        type RoamingDeviceProfileDevEUI = Vec<u8>;
        type RoamingDeviceProfileJoinEUI = Vec<u8>;
        type RoamingDeviceProfileVendorID = Vec<u8>;
    }

    //type System = system::Module<Test>;
    type Balances = balances::Module<Test>;
    type RoamingOperatorTestModule = RoamingOperatorModule<Test>;
    type RoamingNetworkTestModule = RoamingNetworkModule<Test>;
    type RoamingOrganizationTestModule = RoamingOrganizationModule<Test>;
    type RoamingNetworkServerTestModule = RoamingNetworkServerModule<Test>;
    type RoamingAgreementPolicyTestModule = RoamingAgreementPolicyModule<Test>;
    type RoamingAccountingPolicyTestModule = RoamingAccountingPolicyModule<Test>;
    type RoamingRoutingProfileTestModule = RoamingRoutingProfileModule<Test>;
    type RoamingDeviceTestModule = RoamingDeviceModule<Test>;
    type RoamingServiceProfileTestModule = RoamingServiceProfileModule<Test>;
    type RoamingBillingPolicyTestModule = RoamingBillingPolicyModule<Test>;
    type RoamingChargingPolicyTestModule = RoamingChargingPolicyModule<Test>;
    type RoamingNetworkProfileTestModule = RoamingNetworkProfileModule<Test>;
    type RoamingDeviceProfileTestModule = RoamingDeviceProfileModule<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
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
            assert!(RoamingNetworkTestModule::is_roaming_network_owner(0, 1).is_err(), "Roaming network does not exist yet");
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
            assert_ok!(
                RoamingAccountingPolicyTestModule::assign_accounting_policy_to_network(
                    Origin::signed(0),
                    0,
                    0
                )
            );
            assert_eq!(RoamingAccountingPolicyTestModule::roaming_accounting_policy_owner(0), Some(0));
            assert_ok!(
                RoamingAccountingPolicyTestModule::set_config(
                    Origin::signed(0),
                    0, // accounting_policy_id
                    Some("subscription".as_bytes().to_vec()), // policy_type
                    Some(200), // subscription_fee
                    Some(15), // uplink_fee_factor
                    Some(10), // downlink_fee_factor
                )
            );

            // Verify Storage
            assert_eq!(RoamingAccountingPolicyTestModule::roaming_accounting_policies_count(), 1);
            assert_eq!(
                RoamingAccountingPolicyTestModule::roaming_accounting_policy_configs(0),
                Some(RoamingAccountingPolicyConfig {
                    policy_type: "subscription".as_bytes().to_vec(), // policy_type
                    subscription_fee: 200, // subscription_fee
                    uplink_fee_factor: 15, // uplink_fee_factor
                    downlink_fee_factor: 10, // downlink_fee_factor
                })
            );

            // Create Roaming Agreement Policy

            // Call Functions
            assert_ok!(RoamingAgreementPolicyTestModule::create(Origin::signed(0)));
            // Note: This step is optional since it will be assigned to a network when
            // a associated with a network (roaming base) profile 
            assert_ok!(
                RoamingAgreementPolicyTestModule::assign_agreement_policy_to_network(
                    Origin::signed(0),
                    0,
                    0
                )
            );
            // assert_eq!(
            //     RoamingAgreementPolicyTestModule::exists_roaming_agreement_policy(0),
            //     Ok(RoamingAgreementPolicy([0; 16]))
            // );
            assert_eq!(RoamingAgreementPolicyTestModule::roaming_agreement_policy_owner(0), Some(0));
            assert_ok!(
                RoamingAgreementPolicyTestModule::set_config(
                    Origin::signed(0),
                    0,
                    Some("passive".as_bytes().to_vec()),
                    Some(2019)
                )
            );

            // Verify Storage
            assert_eq!(RoamingAgreementPolicyTestModule::roaming_agreement_policies_count(), 1);
            assert_eq!(
                RoamingAgreementPolicyTestModule::roaming_agreement_policy_configs(0),
                Some(RoamingAgreementPolicyConfig {
                    policy_activation_type: "passive".as_bytes().to_vec(),
                    policy_expiry: 2019,
                })
            );

            // Create Roaming Routing Profile

            // Call Functions
            assert_ok!(RoamingRoutingProfileTestModule::create(Origin::signed(0)));
            assert_eq!(RoamingRoutingProfileTestModule::roaming_routing_profile_owner(0), Some(0));
            assert_ok!(
                RoamingRoutingProfileTestModule::set_app_server(
                    Origin::signed(0),
                    0, // routing_profile_id
                    Some("10.0.0.1".as_bytes().to_vec()), // app server
                )
            );

            // Verify Storage
            assert_eq!(RoamingRoutingProfileTestModule::roaming_routing_profiles_count(), 1);
            assert_eq!(
                RoamingRoutingProfileTestModule::roaming_routing_profile_app_server(0),
                Some("10.0.0.1".as_bytes().to_vec())
            );

            // Create Service Profile

            // Call Functions
            assert_ok!(RoamingServiceProfileTestModule::create(Origin::signed(0)));
            assert_eq!(RoamingServiceProfileTestModule::roaming_service_profile_owner(0), Some(0));
            // Note: Optional since it will be assigned to a network when
            // a associated with a network (roaming base) profile, but we can override it to apply to specific
            // network server this way.
            assert_ok!(
                RoamingServiceProfileTestModule::assign_service_profile_to_network_server(
                    Origin::signed(0),
                    0,
                    0
                )
            );
            assert_ok!(
                RoamingServiceProfileTestModule::set_uplink_rate(
                    Origin::signed(0),
                    0, // service_profile_id
                    Some(10), // uplink_rate
                )
            );
            assert_ok!(
                RoamingServiceProfileTestModule::set_downlink_rate(
                    Origin::signed(0),
                    0, // service_profile_id
                    Some(5), // downlink_rate
                )
            );

            // Verify Storage
            assert_eq!(RoamingServiceProfileTestModule::roaming_service_profiles_count(), 1);
            assert_eq!(
                RoamingServiceProfileTestModule::roaming_service_profile_uplink_rate(0),
                Some(10)
            );
            assert_eq!(
                RoamingServiceProfileTestModule::roaming_service_profile_downlink_rate(0),
                Some(5)
            );

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
            assert_ok!(
                RoamingBillingPolicyTestModule::set_config(
                    Origin::signed(0),
                    0,
                    Some(102020), // next_billing_at
                    Some(30) // frequency_in_days
                )
            );

            // Verify Storage
            assert_eq!(RoamingBillingPolicyTestModule::roaming_billing_policies_count(), 1);
            assert_eq!(
                RoamingBillingPolicyTestModule::roaming_billing_policy_configs(0),
                Some(RoamingBillingPolicyConfig {
                    policy_next_billing_at: 102020,
                    policy_frequency_in_days: 30,
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
            assert_ok!(
                RoamingChargingPolicyTestModule::set_config(
                    Origin::signed(0),
                    0,
                    Some(102020), // next_charging_at
                    Some(7) // frequency_in_days
                )
            );

            // Verify Storage
            assert_eq!(RoamingChargingPolicyTestModule::roaming_charging_policies_count(), 1);
            assert_eq!(
                RoamingChargingPolicyTestModule::roaming_charging_policy_configs(0),
                Some(RoamingChargingPolicyConfig {
                    policy_next_charging_at: 102020,
                    policy_delay_after_billing_in_days: 7,
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
            assert_ok!(
                RoamingNetworkProfileTestModule::add_whitelisted_network(
                    Origin::signed(0),
                    0, // network_profile_id
                    0, // network_id
                )
            );
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
            assert_ok!(
                RoamingDeviceProfileTestModule::assign_device_profile_to_device(
                    Origin::signed(0),
                    0,
                    0
                )
            );
            assert_eq!(RoamingDeviceProfileTestModule::roaming_device_profile_owner(0), Some(0));
            assert_ok!(
                RoamingDeviceProfileTestModule::set_config(
                    Origin::signed(0),
                    0,
                    Some("1234".as_bytes().to_vec()), // device_profile_devaddr
                    Some("5678".as_bytes().to_vec()), // device_profile_deveui
                    Some("6789".as_bytes().to_vec()), // device_profile_joineui
                    Some("1000".as_bytes().to_vec()), // device_profile_vendorid
                )
            );

            // Verify Storage
            assert_eq!(RoamingDeviceProfileTestModule::roaming_device_profiles_count(), 1);
            assert_eq!(
                RoamingDeviceProfileTestModule::roaming_device_profile_configs(0),
                Some(RoamingDeviceProfileConfig {
                    device_profile_devaddr: "1234".as_bytes().to_vec(),
                    device_profile_deveui: "5678".as_bytes().to_vec(),
                    device_profile_joineui: "6789".as_bytes().to_vec(),
                    device_profile_vendorid: "1000".as_bytes().to_vec(),
                })
            );
        });
    }
}
