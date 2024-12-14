use crate::system::{
   System,
   load_memory,
   write_memory,
   BlockAllocator,
   MemPermission,
   AccessPermission,
   Mpu,
   ArmException,
};

#[test]
pub fn should_err_on_unaligned_read(){
   let system = System::create(100);
   //println!("{}",system.memory.len());

   let ld_fault_32b = load_memory::<4>(&system, 33);
   let ld_fault_16b = load_memory::<2>(&system, 67);
   let ld_valid_32b = load_memory::<4>(&system, 20);
   let ld_valid_16b = load_memory::<2>(&system, 22);
   let ld_valid_8b = load_memory::<1>(&system, 33);


   assert!(ld_fault_32b.is_err());
   assert!(ld_fault_16b.is_err());
   assert!(ld_valid_32b.is_ok());
   assert!(ld_valid_16b.is_ok());
   assert!(ld_valid_8b.is_ok());
}

#[test]
fn should_err_on_unaligned_write(){
   let mut system = System::create(100);

   let w_fault_32b = write_memory::<4>(&mut system, 33, [1;4]);
   let w_fault_16b = write_memory::<2>(&mut system, 67, [2;2]);
   let w_valid_32b = write_memory::<4>(&mut system, 20, [3;4]);
   let w_valid_16b = write_memory::<2>(&mut system, 22, [5;2]);
   let w_valid_8b = write_memory::<1>(&mut system, 33, [7;1]);

   assert!(w_fault_32b.is_err());
   assert!(w_fault_16b.is_err());
   assert!(w_valid_32b.is_ok());
   assert!(w_valid_16b.is_ok());
   assert!(w_valid_8b.is_ok());
}

#[test]
fn allocator_test(){
   use crate::system::PAGE_SIZE;
   let mut allocator = BlockAllocator::create();
   
   assert_eq!(allocator.get::<4>(104),[0,0,0,0]);
   assert_eq!(allocator.get::<2>(3000),[0,0]);

   allocator.put::<4>(22, [31,2,1,8]);
   assert_eq!(allocator.get::<4>(22),[31,2,1,8]);
   assert_eq!(allocator.pages(),1);

   allocator.put::<1>(PAGE_SIZE as u32 - 1, [8]);
   assert_eq!(allocator.get::<1>(PAGE_SIZE as u32 - 1),[8]);
   assert_eq!(allocator.pages(),1);

   allocator.put::<4>(PAGE_SIZE as u32 - 200, [51,51,51,51]);
   assert_eq!(allocator.get::<4>(PAGE_SIZE as u32 - 200),[51,51,51,51]);
   assert_eq!(allocator.pages(),1);
   
   allocator.put::<4>(PAGE_SIZE as u32 - 4, [2,2,2,2]);
   assert_eq!(allocator.get::<4>(PAGE_SIZE as u32 - 4),[2,2,2,2]);
   assert_eq!(allocator.pages(),1);

   allocator.put::<2>(3000,[87,41]);
   assert_eq!(allocator.get::<2>(3000),[87,41]);
   assert_eq!(allocator.pages(),2);

   let big_address =(PAGE_SIZE * 100) as u32 + 232;
   allocator.put::<4>(big_address,[50,42,99,200]);
   assert_eq!(allocator.get::<4>(big_address),[50,42,99,200]);
   assert_eq!(allocator.pages(),3);

   let big_address_b =(PAGE_SIZE * 200) as u32 + 42;
   allocator.put::<4>(big_address_b,[11,22,33,44]);
   assert_eq!(allocator.get::<4>(big_address_b),[11,22,33,44]);
   assert_eq!(allocator.pages(),4);

   //sanity check
   assert_eq!(allocator.get::<4>(22),[31,2,1,8]);
   assert_eq!(allocator.get::<2>(3000),[87,41]);
   assert_eq!(allocator.get::<4>(big_address),[50,42,99,200]);

   //should be able to overwrite memory
   allocator.put::<2>(big_address,[66,3]);
   assert_eq!(allocator.get::<4>(big_address),[66,3,99,200]);
   assert_eq!(allocator.pages(),4);

   allocator.put::<2>(PAGE_SIZE as u32 - 2, [9,8]);
   allocator.put::<2>(PAGE_SIZE as u32,[6,7]);
   assert_eq!(allocator.get_instr_32b(PAGE_SIZE as u32 - 2),[9,8,6,7],"can fetch 32 bit instructions ");

   assert_eq!(allocator.view(PAGE_SIZE as u32 - 2, PAGE_SIZE as u32 + 1),vec![9,8,6,7]);
   assert_eq!(allocator.view(PAGE_SIZE as u32 - 2, PAGE_SIZE as u32 + 1),vec![9,8,6,7]);
}

#[test]
pub fn should_allocate_sections(){
   let a_data = vec!['A' as u8; 2439];
   let a_start: u32 = 33;

   let b_data = vec!['B' as u8; 701];
   let b_start: u32 = 5000;

   let c_data = vec!['C' as u8; 5111];
   let c_start: u32 = 10000;

   let sections = vec![
      (String::from("section A"),a_start,a_data.clone()),
      (String::from("section B"),b_start,b_data.clone()),
      (String::from("section C"),c_start,c_data.clone())
   ];

   let sys = System::with_sections(sections);

   let a_res = sys.alloc.view(a_start, a_start + a_data.len() as u32 - 1);
   let b_res = sys.alloc.view(b_start, b_start + b_data.len() as u32 - 1);
   let c_res = sys.alloc.view(c_start, c_start + c_data.len() as u32 - 1);
   println!("a: {} == {} ?",a_res.len(),a_data.len());
   for i in 0 .. a_res.len(){
      assert_eq!(a_res[i],a_data[i],"{} != {} mismatch at {} ( addr {} in alloccator) ",a_res[i],a_data[i],i, i + a_start as usize);
   }
   println!("b: {} == {} ?",b_res.len(),b_data.len());
   println!("c: {} == {} ?",c_res.len(),c_data.len());
   assert_eq!(a_res,a_data);
   assert_eq!(b_res,b_data);
   assert_eq!(c_res,c_data);
}

#[test]
pub fn memory_permissions_test(){
   let sys = System::create(10);
   let (_,xn) =  sys.default_permissions(0x00000000);
   assert_eq!(xn,false);
   let (_,xn) =  sys.default_permissions(0x1FFFFFFF);
   assert_eq!(xn,false);
   let (_,xn) =  sys.default_permissions(0x20000000);
   assert_eq!(xn,false);
   let (_,xn) =  sys.default_permissions(0x3FFFFFFF);
   assert_eq!(xn,false);

   let (_,xn) =  sys.default_permissions(0x60000000);
   assert_eq!(xn,false);
   let (_,xn) =  sys.default_permissions(0x7FFFFFFF);
   assert_eq!(xn,false);

   let (_,xn) =  sys.default_permissions(0x80000000);
   assert_eq!(xn,false);
   let (_,xn) =  sys.default_permissions(0x9FFFFFFF);
   assert_eq!(xn,false);

   let (_,xn) =  sys.default_permissions(0x40000000);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0x5FFFFFFF);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0xA0000000);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0xBFFFFFFF);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0xC0000000);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0xDFFFFFFF);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0xE0000000);
   assert_eq!(xn,true);
   let (_,xn) =  sys.default_permissions(0xFFFFFFFF);
   assert_eq!(xn,true);
}

macro_rules! bg_perm {
   () => (
      (
         MemPermission{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::ReadAndWrite,
         },
         true
      )
   )
}

#[test]
pub fn basic_mpu_test(){
   let mut mpu = Mpu::reset();
   assert!(mpu.enabled() == false);

   mpu.set_ctrl(1);
   assert!(mpu.enabled());

   //MPU Region 0: 
   //from 0 <-> 256
   //priv: RW, unpriv: R0,
   //subregion size = 256 / 8 = 32
   //subregion 64 <-= 95 is disabled and so should fault on any access 
   
   
   set_permission(
      &mut mpu,
      &Asr{
         xn: true,
         perm: MemPermission{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::ReadOnly,
         },
         size: 7,//256 bytes
         srd: 0b100, // 3rd subregion is disabled
         enable: true,
      }
   ).unwrap();
   let _ = mpu.set_rbar(0).unwrap();

   let (p,px) = mpu.get_permissions(96,false,bg_perm!()).unwrap();
   assert!(
      matches!(
         p,
         MemPermission{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::ReadOnly,
      })
   );
   assert!(px);

   assert!(mpu.get_permissions(192389,false,bg_perm!()).is_err());//unprivileged access
   assert!(mpu.get_permissions(192389,true,bg_perm!()).is_err());//privileged access
   assert!(mpu.get_permissions(256,true,bg_perm!()).is_err());
   assert!(mpu.get_permissions(256,false,bg_perm!()).is_err());

   mpu.set_ctrl(0b101);
   assert!(mpu.get_permissions(95,false,bg_perm!()).is_err());//unprivileged access
   let res = mpu.get_permissions(95,true,bg_perm!()).unwrap(); //privileged access
   assert!(matches!(res,bg_perm!()));
} 

macro_rules! region_perm {
   ($prv:path,$unpriv:path,$xn:literal) => (
      (
         MemPermission{
            privileged: $prv,
            unprivileged: $unpriv,
         },
         $xn
      )
   )
}
#[test]
pub fn mpu_multiple_regions(){
   let mut mpu = Mpu::reset();
   mpu.set_ctrl(1);
   //region 0 
   //256 -> 512: priv:RW, unpriv:NA | XN (size: 1 << 8)
   //
   //region 1
   //4096 -> 8192: priv:RO unpriv: RO | !XN (size: 1 << 12)
   //
   //region 2
   //6144 -> 6656: priv:NA priv: NA | XN (size: 1 << 9)
   let (perm_r0,xn_r0) = region_perm!(
      AccessPermission::ReadAndWrite,
      AccessPermission::NoAccess,
      true
   );
   mpu.set_rnr(0).unwrap();
   set_permission(
      &mut mpu,
      &Asr{
         xn: xn_r0,
         perm: perm_r0.clone(),
         size: 7, 
         srd: 0,
         enable: true,
      }
   ).unwrap();
   let _ = mpu.set_rbar(256 << 8).unwrap();

   let (perm_r1,xn_r1) = region_perm!(
      AccessPermission::ReadOnly,
      AccessPermission::ReadOnly,
      false
   );
   mpu.set_rnr(1).unwrap();
   set_permission(
      &mut mpu,
      &Asr{
         xn: xn_r1,
         perm: perm_r1.clone(),
         size: 11, 
         srd: 0,
         enable: true,
      }
   ).unwrap();
   let _ = mpu.set_rbar(4096 << 8).unwrap();

   let (perm_r2,xn_r2) = region_perm!(
      AccessPermission::NoAccess,
      AccessPermission::NoAccess,
      true
   );
   mpu.set_rnr(2).unwrap();
   set_permission(
      &mut mpu,
      &Asr{
         xn: xn_r2,
         perm: perm_r2.clone(),
         size: 8, 
         srd: 0,
         enable: true,
      }
   ).unwrap();
   let _ = mpu.set_rbar(6144 << 8).unwrap();

   //256 -> 512: priv:RW, unpriv:NA | XN (size: 1 << 8)
   //4096 -> 8192: priv:RO unpriv: RO | !XN (size: 1 << 12)
   //6144 -> 6656: priv:NA priv: NA | XN (size: 1 << 9)
   assert!(matches!(mpu.get_permissions(300,false,bg_perm!()).unwrap().0, perm_r0));
   assert_eq!(mpu.get_permissions(300,false,bg_perm!()).unwrap().1, xn_r0);

   assert!(matches!(mpu.get_permissions(7123,false,bg_perm!()).unwrap().0, perm_r1));
   assert_eq!(mpu.get_permissions(7123,false,bg_perm!()).unwrap().1, xn_r1);

   //when regions overlap the region with the higher number takes priority
   assert!(matches!(mpu.get_permissions(6650,false,bg_perm!()).unwrap().0, perm_r2));
   assert_eq!(mpu.get_permissions(6650,false,bg_perm!()).unwrap().1, xn_r2);

   assert!(mpu.get_permissions(100000,false,bg_perm!()).is_err());
}

#[test]
pub fn mpu_enforce_region_alignment(){
   let mut mpu = Mpu::reset();
   mpu.set_ctrl(1);
   
   //MPU Region 0: 
   //from 256 <-> 512
   //priv: RW, unpriv: R0,
   //subregion size = 256 / 8 = 32
   mpu.set_rnr(0).unwrap();
   set_permission(
      &mut mpu,
      &Asr{
         xn: true,
         perm: MemPermission{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::ReadOnly,
         },
         size: 7,//256 bytes
         srd: 0, // 3rd subregion is disabled
         enable: true,
      }
   ).unwrap();
   let _ = mpu.set_rbar(267 << 8).unwrap();

   assert_eq!(mpu.get_rbar(), 256 << 8);
   let (p,px) = mpu.get_permissions(256,false,bg_perm!()).unwrap();
   assert!(
      matches!(
         p,
         MemPermission{
            privileged: AccessPermission::ReadAndWrite,
            unprivileged: AccessPermission::ReadOnly,
         }
      )
   );
}

struct Asr{
   pub xn: bool, 
   pub perm: MemPermission,
   pub size: u32,
   pub srd: u32,
   pub enable: bool,
}

fn set_permission(mpu: &mut Mpu, asr: &Asr)->Result<(),ArmException>{

   let attr: u32 = match (asr.perm.privileged, asr.perm.unprivileged){
      (AccessPermission::NoAccess,AccessPermission::NoAccess) => 0b000,
      (AccessPermission::ReadAndWrite,AccessPermission::NoAccess) => 0b001,
      (AccessPermission::ReadAndWrite,AccessPermission::ReadOnly) => 0b010,
      (AccessPermission::ReadAndWrite,AccessPermission::ReadAndWrite) => 0b011,
      (AccessPermission::ReadOnly,AccessPermission::NoAccess) => 0b100,
      (AccessPermission::ReadOnly,AccessPermission::ReadOnly) => 0b111,
      _ => unreachable!()
   };

   let rasr = ((asr.xn as u32) << 28)
      | ((asr.size & 0x1F) << 1)
      | (asr.enable as u32) 
      | (asr.srd << 8)
      | (attr << 24);


   println!("raw rasr: {:#x}",rasr);
   let _ = mpu.set_rasr(rasr)?;

   return Ok(());
}
