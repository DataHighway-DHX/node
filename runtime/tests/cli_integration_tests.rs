// extern crate env as env;
extern crate roaming_operators as operators;
extern crate roaming_networks as networks;
extern crate roaming_organizations as organizations;
extern crate roaming_network_servers as network_servers;
extern crate roaming_agreement_policies as agreement_policies;
extern crate roaming_accounting_policies as accounting_policies;
extern crate roaming_routing_profiles as routing_profiles;
extern crate roaming_devices as devices;

#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{H256};
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_noop, assert_ok, impl_outer_origin, parameter_types, weights::Weight};
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
        // RoamingRoutingProfileAppServer,
        Trait as RoamingRoutingProfileTrait,
    };
    use roaming_devices::{
        Module as RoamingDeviceModule,
        RoamingDevice,
        Trait as RoamingDeviceTrait,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
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
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ();
        type TransferFee = ();
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
    type Randomness = randomness_collective_flip::Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
        let mut t = system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        balances::GenesisConfig::<Test> {
            balances: vec![(1, 10), (2, 20), (3, 30)],
            vesting: vec![],
        }
        .assimilate_storage(&mut t)
        .unwrap();
        runtime_io::TestExternalities::new(t)
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
            // FIXME - assign Users to the organizations

            // Verify Storage
            assert_eq!(RoamingOrganizationTestModule::roaming_organizations_count(), 2);

            // Create Network Servers
            //
            // Create a network server with the same admin user.

            // Call Functions
            assert_ok!(RoamingNetworkServerTestModule::create(Origin::signed(0)));

            // Verify Storage
            assert_eq!(RoamingNetworkServerTestModule::roaming_network_servers_count(), 1);

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
            // Note: This step is optional
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
        });
    }
}
