# Clone代码

新建创建项目的`clone.sh`，然后`bash clone.sh`自动创建项目

```bash
#!/bin/bash

BRANCH="debin/2vm_timer"

mkdir -p crates

git clone $BRANCH https://github.com/arceos-hypervisor/arceos-umhv.git 

cd arceos-umhv

# 克隆arceos主仓库
git clone $BRANCH https://github.com/arceos-hypervisor/arceos.git ../arceos

# 克隆其他仓库到crates目录
REPOS=(
    "axvm"
    "axvcpu"
    "axaddrspace"
    "arm_vcpu"
    "axdevice"
    "arm_vgic"
    "arm_gicv2"
    "axdevice_crates"
)

for repo in "${REPOS[@]}"; do
    git clone $BRANCH "https://github.com/arceos-hypervisor/${repo}.git" "../crates/${repo}"
done

echo "所有仓库克隆完成！"

# 创建临时文件
temp_file=$(mktemp)

# 要添加的新内容
cat > "$temp_file" << 'EOF'
[patch."https://github.com/arceos-hypervisor/arceos.git".axstd]
path = "../arceos/ulib/axstd"
[patch."https://github.com/arceos-hypervisor/arceos.git".axhal]
path = "../arceos/modules/axhal"
[patch."https://github.com/arceos-hypervisor/axvm.git".axvm]
path = "../crates/axvm"
[patch."https://github.com/arceos-hypervisor/axvcpu.git".axvcpu]
path = "../crates/axvcpu"
[patch."https://github.com/arceos-hypervisor/axaddrspace.git".axaddrspace]
path = "../crates/axaddrspace"
[patch."https://github.com/arceos-hypervisor/arm_vcpu.git".arm_vcpu]
path = "../crates/arm_vcpu"
[patch."https://github.com/arceos-hypervisor/axdevice.git".axdevice]
path = "../crates/axdevice"
[patch."https://github.com/arceos-hypervisor/arm_vgic.git".arm_vgic]
path = "../crates/arm_vgic"
[patch."https://github.com/arceos-hypervisor/axdevice_crates.git".axdevice_base]
path = "../crates/axdevice_crates/axdevice_base"
[patch."https://github.com/arceos-hypervisor/arm_gicv2.git".arm_gicv2]
path = "../crates/arm_gicv2"

EOF

# 将原文件内容追加到临时文件
cat Cargo.toml >> "$temp_file"

# 将临时文件移回原文件
mv "$temp_file" Cargo.toml

echo "成功更新 Cargo.toml"

cd .. && mkdir .vscode

cat > .vscode/settings.json << 'EOF'
{
    "rust-analyzer.cargo.target": "aarch64-unknown-none-softfloat",
    "rust-analyzer.check.allTargets": false,
    "rust-analyzer.cargo.features": ["irq", "hv"],
    "rust-analyzer.cargo.extraEnv": {
        "RUSTFLAGS": "--cfg platform_family=\"aarch64-qemu-virt\""
    }
}
EOF

```



# 编译nimbos

因为加载到任意地址的功能还没有实现，所以只能通过硬配置来做，得单独编译两个nimbos

## nimbos（VM1）

```bash
git clone https://github.com/arceos-hypervisor/nimbos.git
```

## nimbos（VM2）

```bash
git clone -b debin/0x800 https://github.com/arceos-hypervisor/nimbos.git
```



# 创建`disk.img`文件

生成一个disk.img，然后将编译好的nimbos.bin重命名并放入里面

```bash
cd arceos-umhv/arceos-vmm/
mkdir mnt
make disk_img
sudo mount disk_img mnt
cp nimbos_0x408_0000.bin ./mnt
cp nimbos_0x808_0000.bin ./mnt
cd .. && sudo umount mnt
```



# 启动VMM

```bash
cd arceos-umhv/arceos-vmm/
bash run.sh

# 在qemu启动后，打开第二个终端使用telnet连接串口2
telnet localhost 4321
```

就可以正常注入timer了

## VM1

```bash
NN   NN  iii               bb        OOOOO    SSSSS
NNN  NN       mm mm mmmm   bb       OO   OO  SS
NN N NN  iii  mmm  mm  mm  bbbbbb   OO   OO   SSSSS
NN  NNN  iii  mmm  mm  mm  bb   bb  OO   OO       SS
NN   NN  iii  mmm  mm  mm  bbbbbb    OOOO0    SSSSS
              ___    ____    ___    ___
             |__ \  / __ \  |__ \  |__ \
             __/ / / / / /  __/ /  __/ /
            / __/ / /_/ /  / __/  / __/
           /____/ \____/  /____/ /____/

arch = aarch64
platform = qemu-virt-arm
build_mode = release
log_level = info

Initializing kernel heap at: [0xffff0000401200e0, 0xffff0000405200e0)
[INFO  nimbos] Logging is enabled.
Initializing frame allocator at: [PA:0x40521000, PA:0x48000000)
Mapping .text: [0xffff000040080000, 0xffff000040094000)
Mapping .rodata: [0xffff000040094000, 0xffff00004009b000)
Mapping .data: [0xffff00004009b000, 0xffff00004011a000)
Mapping .bss: [0xffff00004011e000, 0xffff000040521000)
Mapping boot stack: [0xffff00004011a000, 0xffff00004011e000)
Mapping physical memory: [0xffff000040521000, 0xffff000048000000)
[  0.280129 1:9 arceos_vmm::vmm::vcpus:243] VM[2] Vcpu[0] waiting for running
[  0.280591 1:9 arceos_vmm::vmm::vcpus:246] VM[2] Vcpu[0] running...
Mapping MMIO: [0xffff000009000000, 0xffff000009001000)
Mapping MMIO: [0xffff000008000000, 0xffff000008020000)
Initializing drivers...
[  0.291414 0:8 arm_vgic::interrupt:76] Setting interrupt 30 enable to true
Initializing task manager...
/**** APPS ****
cyclictest
exit
fantastic_text
forktest
forktest2
forktest_simple
forktest_simple_c
forktree
hello_c
hello_world
matrix
poweroff
sleep
sleep_simple
stack_overflow
thread_simple
user_shell
usertests
yield
**************/
Running tasks...
test kernel task: pid = TaskId(2), arg = 0xdead
[  0.294993 INFO  nimbos::task::structs][0:2] task exit with code 0
test kernel task: pid = TaskId(3), arg = 0xbeef
[  0.296126 INFO  nimbos::task::structs][0:3] task exit with code 0
[  0.296457 INFO  nimbos::arch::aarch64::context][0:4] user task start: entry=0x211cfc, ustack=0xfffffffff000, kstack=0xffff000040138000
Rust user shell
>> [  0.298106 1:9 arm_vgic::interrupt:76] Setting interrupt 30 enable to true
[  3.349047 0:8 axhal::irq:23] Unhandled IRQ 33
s[  3.583202 0:8 axhal::irq:23] Unhandled IRQ 33
l[  3.898013 0:8 axhal::irq:23] Unhandled IRQ 33
e[  4.067715 0:8 axhal::irq:23] Unhandled IRQ 33
e[  4.214772 0:8 axhal::irq:23] Unhandled IRQ 33
p[  4.829631 0:8 axhal::irq:23] Unhandled IRQ 33

[  4.830637 INFO  nimbos::arch::aarch64::context][0:5] user task start: entry=0x211d28, ustack=0xffffffffee10, kstack=0xffff000040134000
[  4.832135 INFO  nimbos::arch::aarch64::context][0:6] user task start: entry=0x210744, ustack=0xffffffffef40, kstack=0xffff000040130000
sleep 1 x 1 seconds.
sleep 2 x 1 seconds.
sleep 3 x 1 seconds.
sleep 4 x 1 seconds.
sleep 5 x 1 seconds.
[  9.879593 INFO  nimbos::task::structs][0:6] task exit with code 0
use 5048222 usecs.
sleep passed!
[  9.880587 INFO  nimbos::task::structs][0:5] task exit with code 0
Shell: Process 5 exited with code 0
>> QEMU: Terminated
```



## VM2

```bash
Trying 127.0.0.1...
Connected to localhost.
Escape character is '^]'.
a
NN   NN  iii               bb        OOOOO    SSSSS
NNN  NN       mm mm mmmm   bb       OO   OO  SS
NN N NN  iii  mmm  mm  mm  bbbbbb   OO   OO   SSSSS
NN  NNN  iii  mmm  mm  mm  bb   bb  OO   OO       SS
NN   NN  iii  mmm  mm  mm  bbbbbb    OOOO0    SSSSS
              ___    ____    ___    ___
             |__ \  / __ \  |__ \  |__ \
             __/ / / / / /  __/ /  __/ /
            / __/ / /_/ /  / __/  / __/
           /____/ \____/  /____/ /____/

arch = aarch64
platform = qemu-virt-arm
build_mode = release
log_level = info

Initializing kernel heap at: [0xffff0000801200e0, 0xffff0000805200e0)
[INFO  nimbos] Logging is enabled.
Initializing frame allocator at: [PA:0x80521000, PA:0x88000000)
Mapping .text: [0xffff000080080000, 0xffff000080094000)
Mapping .rodata: [0xffff000080094000, 0xffff00008009b000)
Mapping .data: [0xffff00008009b000, 0xffff00008011a000)
Mapping .bss: [0xffff00008011e000, 0xffff000080521000)
Mapping boot stack: [0xffff00008011a000, 0xffff00008011e000)
Mapping physical memory: [0xffff000080521000, 0xffff000088000000)
Mapping MMIO: [0xffff000009000000, 0xffff000009001000)
Mapping MMIO: [0xffff000008000000, 0xffff000008020000)
Initializing drivers...
Initializing task manager...
/**** APPS ****
cyclictest
exit
fantastic_text
forktest
forktest2
forktest_simple
forktest_simple_c
forktree
hello_c
hello_world
matrix
poweroff
sleep
sleep_simple
stack_overflow
thread_simple
user_shell
usertests
yield
**************/
Running tasks...
test kernel task: pid = TaskId(2), arg = 0xdead
[  0.301555 INFO  nimbos::task::structs][0:2] task exit with code 0
test kernel task: pid = TaskId(3), arg = 0xbeef
[  0.302448 INFO  nimbos::task::structs][0:3] task exit with code 0
[  0.302702 INFO  nimbos::arch::aarch64::context][0:4] user task start: entry=0x211cfc, ustack=0xfffffffff000, kstack=0xffff000080138000
Rust user shell
>> sleep
[ 13.410960 INFO  nimbos::arch::aarch64::context][0:5] user task start: entry=0x211d28, ustack=0xffffffffee10, kstack=0xffff000080134000
[ 13.412543 INFO  nimbos::arch::aarch64::context][0:6] user task start: entry=0x210744, ustack=0xffffffffef40, kstack=0xffff000080130000
sleep 1 x 1 seconds.
sleep 2 x 1 seconds.
sleep 3 x 1 seconds.
sleep 4 x 1 seconds.
sleep 5 x 1 seconds.
[ 18.460517 INFO  nimbos::task::structs][0:6] task exit with code 0
use 5048755 usecs.
sleep passed!
[ 18.461516 INFO  nimbos::task::structs][0:5] task exit with code 0
Shell: Process 5 exited with code 0
>> Connection closed by foreign host.
```

