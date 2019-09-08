const fs = require('fs');
const path = require('path');
const dotenv = require('dotenv');
const {Enigma, utils, eeConstants} = require('enigma-js/node');
const EthCrypto = require('eth-crypto')

var EnigmaContract;
if(typeof process.env.SGX_MODE === 'undefined' || (process.env.SGX_MODE != 'SW' && process.env.SGX_MODE != 'HW' )) {
    console.log(`Error reading ".env" file, aborting....`);
    process.exit();
} else if (process.env.SGX_MODE == 'SW') {
  EnigmaContract = require('../build/enigma_contracts/EnigmaSimulation.json');
} else {
  EnigmaContract = require('../build/enigma_contracts/Enigma.json');
}
const EnigmaTokenContract = require('../build/enigma_contracts/EnigmaToken.json');


function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

let enigma = null;
let contractAddr;

contract("Enable", accounts => {
  before(function() {
    enigma = new Enigma(
      web3,
      EnigmaContract.networks['4447'].address,
      EnigmaTokenContract.networks['4447'].address,
      'http://localhost:3346',
      {
        gas: 4712388,
        gasPrice: 100000000000,
        from: accounts[0],
      },
    );
    enigma.admin();

    contractAddr = fs.readFileSync('test/enable.txt', 'utf-8');
  });

  let task;

  // ADDING USER
  it('should execute compute task to add a user', async () => {
    let taskFn = 'add_user(string,bytes)';
    let taskArgs = [
      ['1', 'string'],
      [web3.utils.hexToBytes('0x4f3edf983ac636a65a842ce7c78d9aa706d3b113bce9c46f30d7d21715b23b1d'), 'bytes']
    ];
    let taskGasLimit = 500000;
    let taskGasPx = utils.toGrains(1);
    task = await new Promise((resolve, reject) => {
      enigma.computeTask(taskFn, taskArgs, taskGasLimit, taskGasPx, accounts[0], contractAddr)
        .on(eeConstants.SEND_TASK_INPUT_RESULT, (result) => resolve(result))
        .on(eeConstants.ERROR, (error) => reject(error));
    });
  });

  it('should get the pending task', async () => {
    task = await enigma.getTaskRecordStatus(task);
    expect(task.ethStatus).to.equal(1);
  });

  it('should get the confirmed task', async () => {
    do {
      await sleep(1000);
      task = await enigma.getTaskRecordStatus(task);
      process.stdout.write('Waiting. Current Task Status is '+task.ethStatus+'\r');
    } while (task.ethStatus !== 2);
    expect(task.ethStatus).to.equal(2);
    process.stdout.write('Completed. Final Task Status is '+task.ethStatus+'\n');
  }, 10000);

  // ADDING LOGIN
  it('should execute compute task to add a login', async () => {
    let taskFn = 'add_login(string,uint256)';
    let taskArgs = [
      ['1', 'string'],
      [12345, 'uint256']
    ];
    let taskGasLimit = 500000;
    let taskGasPx = utils.toGrains(1);
    task = await new Promise((resolve, reject) => {
      enigma.computeTask(taskFn, taskArgs, taskGasLimit, taskGasPx, accounts[0], contractAddr)
      .on(eeConstants.SEND_TASK_INPUT_RESULT, (result) => resolve(result))
      .on(eeConstants.ERROR, (error) => reject(error));
    });
  });

  it('should get the pending task', async () => {
    task = await enigma.getTaskRecordStatus(task);
    expect(task.ethStatus).to.equal(1);
  });

  it('should get the confirmed task', async () => {
    do {
      await sleep(1000);
      task = await enigma.getTaskRecordStatus(task);
      process.stdout.write('Waiting. Current Task Status is '+task.ethStatus+'\r');
    } while (task.ethStatus !== 2);
    expect(task.ethStatus).to.equal(2);
    process.stdout.write('Completed. Final Task Status is '+task.ethStatus+'\n');
  }, 10000);

  // SIGN MESSAGE
  it('should sign a message', async () => {
    let taskFn = 'sign_message(uint256,string)';
    let taskArgs = [
      [12345, 'uint256'],
      ['Hello world!', 'string']
    ];
    let taskGasLimit = 50000000;
    let taskGasPx = utils.toGrains(1);
    task = await new Promise((resolve, reject) => {
      enigma.computeTask(taskFn, taskArgs, taskGasLimit, taskGasPx, accounts[0], contractAddr)
      .on(eeConstants.SEND_TASK_INPUT_RESULT, (result) => resolve(result))
      .on(eeConstants.ERROR, (error) => reject(error));
    });
  });

  it('should get the pending task', async () => {
    task = await enigma.getTaskRecordStatus(task);
    expect(task.ethStatus).to.equal(1);
  });

  it('should get the confirmed task', async () => {
    do {
      await sleep(1000);
      task = await enigma.getTaskRecordStatus(task);
      process.stdout.write('Waiting. Current Task Status is '+task.ethStatus+'\r');
    } while (task.ethStatus !== 2);
    expect(task.ethStatus).to.equal(2);
    process.stdout.write('Completed. Final Task Status is '+task.ethStatus+'\n');
  }, 10000);

  it('should verify the signature', async () => {
    task = await new Promise((resolve, reject) => {
      enigma.getTaskResult(task)
      .on(eeConstants.GET_TASK_RESULT_RESULT, (result) => resolve(result))
      .on(eeConstants.ERROR, (error) => reject(error));
    });
    expect(task.engStatus).to.equal('SUCCESS');
    task = await enigma.decryptTaskResult(task);
    let result = task.decryptedOutput;
    let pubkey = result.substr(128, 128);
    let signature = '0x' + result.substr(256).replace('00000000000000000000000000000000000000000000000000000000000000', '');
    console.log(pubkey)
    console.log(signature) 
    expect(EthCrypto.recoverPublicKey(signature, EthCrypto.hash.keccak256('Hello world!'))).to.equal(pubkey)
  });
});