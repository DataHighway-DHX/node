// Import the API & Provider and some utility functions
const { ApiPromise, WsProvider } = require('@polkadot/api');

// import the test keyring (already has dev keys for Alice, Bob, Charlie, Eve & Ferdie)
const testKeyring = require('@polkadot/keyring/testing');
const createTestPairs = require('@polkadot/keyring/testingPairs');
const Decimal = require('decimal.js');


const fs = require('fs');

async function main () {
  const wsEndpointDev = 'ws://127.0.0.1:9944'; // Development
  const wsEndpointTestnet = 'ws://testnet-harbour.datahighway.com'; // Testnet
  const wsEndpointMainnet = 'ws://westlake.datahighway.com'; // Mainnet
  // Initialise the provider to connect to a node
  const provider = new WsProvider(wsEndpointDev);

  // Create the API and wait until ready (optional provider passed through)
  const api = await ApiPromise.create({
    provider,
    types: {
      AccountInfo: 'AccountInfoWithDualRefCount'
    }
  });

  // https://polkadot.js.org/docs/keyring/start/create/#adding-a-pair

  // Retrieve the upgrade key from the chain state
//   const sudoId = await api.query.sudo.key();

  // Find the actual keypair in the keyring (if this is a changed value, the key
  // needs to be added to the keyring before - this assumes we have defaults, i.e.
  // Alice as the key - and this already exists on the test keyring)
//   console.log('testKeyring', testKeyring)
//   const keyring = testKeyring.default();
// const keyring =
// console.log('keyring', keyring)
//   const sudoPair = testKeyring.PAIRSSR25519[0];
//   console.log('sudoId', sudoId);
//   console.log('sudoPair', sudoPair);

const pairs = createTestPairs;
const sudoPair = pairs.alice;

//   // TODO - add mnemonics
//   const treasuryMnemonic = '';
//   // create & add the pair to the keyring with the type and some additional
//   // metadata specified. log the name & address (the latter encoded with the ss58Format)
//   const TREASURY_PAIR = keyring.addFromUri(treasuryMnemonic, { name: 'Treasury' }, 'sr25519');
//   console.log(pair.meta.name, 'has address', pair.address);

  // the pair has been added to our keyring
  //   console.log(keyring.pairs.length, 'pairs available');

  const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY'; // subkey inspect //Alice
  const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
  const CHARLIE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';

  // https://github.com/MikeMcl/decimal.js/
  const amount = 1; //new Decimal('1');

  const unsub = await api.tx.sudo
    .sudo(
      api.tx.balances.forceTransfer(BOB, CHARLIE, amount)
    )
    .signAndSend(sudoPair, ({ status, events }) => {
      if (status.isInBlock || status.isFinalized) {
        console.log('Successful Sudo transfer of ' + amount + ' with hash ' + status.asInBlock.toHex());

        console.log('Included at block hash', status.asInBlock.toHex());
        console.log('Finalized block hash', status.asFinalized.toHex());

        events
          // We know this tx should result in `Sudid` event.
          .filter(({ event }) =>
            api.events.sudo.Sudid.is(event)
          )
          // We know that `Sudid` returns just a `Result`
          .forEach(({ event : { data: [result] } }) => {
            // Now we look to see if the extrinsic was actually successful or not...
            if (result.isError) {
              let error = result.asError;
              if (error.isModule) {
                // for module errors, we have the section indexed, lookup
                const decoded = api.registry.findMetaError(error.asModule);
                const { documentation, name, section } = decoded;

                console.log(`${section}.${name}: ${documentation.join(' ')}`);
              } else {
                // Other, CannotLookup, BadOrigin, no extra info
                console.log(error.toString());
              }
            }
          });
        unsub();
      } else {
        console.log('Status of Sudo transfer: ' + status.type);
      }
    });
}

main().catch((error) => {
  console.error(error);
  process.exit(-1);
});
