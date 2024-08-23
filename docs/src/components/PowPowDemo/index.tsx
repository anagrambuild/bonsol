import React, { useState, useEffect, useMemo, useRef } from 'react';
import { Connection } from '@solana/web3.js';
import { UnifiedWalletProvider, UnifiedWalletButton, useWallet } from '@jup-ag/wallet-adapter';
import { PhantomWalletAdapter } from '@solana/wallet-adapter-phantom';
import { SolflareWalletAdapter } from '@solana/wallet-adapter-solflare';
import { BackpackWalletAdapter } from '@solana/wallet-adapter-backpack';
import { NightlyWalletAdapter } from '@solana/wallet-adapter-nightly';
import { UnsafeBurnerWalletAdapter } from '@solana/wallet-adapter-unsafe-burner';
import { gsap } from 'gsap';

function CollatzDemo() {
  const [currentNumber, setCurrentNumber] = useState(null);
  const [sequenceLength, setSequenceLength] = useState(0);
  const [difficulty, setDifficulty] = useState(0);
  const { publicKey, signMessage } = useWallet();
  const [currentSlot, setCurrentSlot] = useState(0);
  const canvasRef = useRef(null);
  const ctxRef = useRef(null);
  const sequenceRef = useRef([]);
  const timelineRef = useRef(null);
  const connection = useMemo(() => new Connection('https://celestine-kww4t3-fast-mainnet.helius-rpc.com'), []);

  useEffect(() => {
    const intervalId = setInterval(async () => {
      try {
        const slot = await connection.getSlot();
        setCurrentSlot(slot);
      } catch (error) {
        console.error('Error fetching slot:', error);
      }
    }, 1000);

    return () => clearInterval(intervalId);
  }, [connection]);

  useEffect(() => {
    if (canvasRef.current) {
      const canvas = canvasRef.current;
      canvas.width = 800;
      canvas.height = 700;
      ctxRef.current = canvas.getContext('2d');
    }
  }, []);

  const slotToLittleEndianUint8Array = (slot) => {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigUint64(0, BigInt(slot), true);
    return new Uint8Array(buffer);
  };

  const generateLargeBigIntFromSignature = async () => {
    try {
      if (!publicKey) {
        throw new Error('Wallet not connected');
      }
      const message = slotToLittleEndianUint8Array(currentSlot);
      const signature = await signMessage(message);
      return BigInt(`0x${Buffer.from(signature).toString('hex')}`);
    } catch (error) {
      console.error('Error generating number from signature:', error);
      return generateRandomLargeBigInt();
    }
  };

  const generateRandomLargeBigInt = () => {
    const digits = Array.from({ length: 155 }, () => Math.floor(Math.random() * 10)).join('');
    return BigInt(digits);
  };

  const collatzStep = (n) => {
    return n % 2n === 0n ? n / 2n : 3n * n + 1n;
  };

  const calculateDifficulty = (sequence) => {
    const sum = sequence.reduce((acc, val) => acc + val, 0n);
    const maxElement = sequence.reduce((max, val) => val > max ? val : max, 0n);
    const difficulty = (Math.log2(Number(sum)) * sequence.length) / Math.log2(Number(maxElement));
    return difficulty.toFixed(2);
  };

  const drawSequence = (progress) => {
    const ctx = ctxRef.current;
    const canvas = canvasRef.current;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    ctx.beginPath();
    ctx.strokeStyle = '#00ff00';
    ctx.lineWidth = 2;

    const centerX = canvas.width / 2;
    const centerY = canvas.height / 2;
    const maxRadius = Math.min(canvas.width, canvas.height) / 2 - 10;

    const currentIndex = Math.floor(progress * sequenceRef.current.length);

    for (let i = 0; i <= currentIndex; i++) {
      const diff = Number(sequenceRef.current[i]) - Number(sequenceRef.current[0]);
      const angle = (i / sequenceRef.current.length ) * Math.PI * (diff - sequenceRef.current.length);
      
      const radius = maxRadius * (Math.log(Number(sequenceRef.current[i])) / Math.log(Number(sequenceRef.current[0])));
      const x = centerX + Math.cos(angle) * radius;
      const y = centerY + Math.sin(angle) * radius;

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    }

    ctx.stroke();

    setSequenceLength(currentIndex + 1);
    setDifficulty(calculateDifficulty(sequenceRef.current.slice(0, currentIndex + 1)));
  };

  const handleGenerateClick = async () => {
    if (timelineRef.current) {
      timelineRef.current.kill();
    }

    const startNumber = publicKey ? await generateLargeBigIntFromSignature() : generateRandomLargeBigInt();
    setCurrentNumber(startNumber);
    sequenceRef.current = [startNumber];
    let current = startNumber;

    while (current !== 1n) {
      current = collatzStep(current);
      sequenceRef.current.push(current);
    }

    timelineRef.current = gsap.timeline();
    timelineRef.current.to({}, {
      duration: 25, // Adjust this value to control the overall animation duration
      onUpdate: function() {
        drawSequence(this.progress());
      },
      ease: "power1.inOut"
    });
  };

  return (
    <div className="relative" style={{display: 'flex', flexDirection: 'column', padding: '20px', backgroundColor: '#000', color: '#00ff00' }}>
      <div style={{ marginBottom: '20px' }}>
        <UnifiedWalletButton />
      </div>
      <div style={{ fontFamily: 'monospace' }}>Current Solana Slot: {currentSlot}</div>
      <button 
        onClick={handleGenerateClick} 
        style={{ 
          marginTop: '10px', 
          marginBottom: '10px', 
          backgroundColor: '#00ff00', 
          color: '#000', 
          border: 'none', 
          padding: '10px', 
          fontFamily: 'monospace', 
          cursor: 'pointer' 
        }}
      >
        Generate Collatz Sequence
      </button>
      <canvas ref={canvasRef} style={{ border: '1px solid #00ff00' }}></canvas>
      {currentNumber && (
        <div style={{ marginTop: '10px', fontFamily: 'monospace' }}>
          <p>Starting number: {currentNumber.toString()}</p>
          <p>Current sequence length: {sequenceLength}</p>
          <p>Difficulty: {difficulty}</p>
        </div>
      )}
    </div>
  );
}

export default function Demo() {
  const params = useMemo(
    () => ({
      wallets: [
        new PhantomWalletAdapter(),
        new SolflareWalletAdapter(),
        new BackpackWalletAdapter(),
        new NightlyWalletAdapter(),
        new UnsafeBurnerWalletAdapter(),
      ],
      config: {
        autoConnect: false,
        env: 'mainnet-beta',
        metadata: {
          name: 'Bonsol Collatz Demo',
          description: 'Bonsol Collatz Sequence Demo',
          url: 'https://bonsol.sh',
          iconUrls: ['https://bonsol.sh/favicon.ico'],
        }
      },
    }),
    []
  );

  return (
    <UnifiedWalletProvider {...params}>
      <CollatzDemo />
    </UnifiedWalletProvider>
  );
}