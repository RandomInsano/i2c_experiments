 // file: ds3231.cpp
#include <stdio.h> // printf()
#include <sys/types.h> // open()
#include <sys/stat.h> // open()
#include <fcntl.h> // open()
#include <sys/ioctl.h> // ioctl()
#include <errno.h> // errno
#include <string.h> // strerror()
#include <unistd.h> // close()
#include <linux/i2c-dev.h> // struct i2c_msg
#include <linux/i2c.h> // struct i2c_rdwr_ioctl_data
int main(int argc, char * argv[])
{
struct i2c_msg ds3231msg[2]; // declare our two i2c_msg array
unsigned char ucIndex = 2;
unsigned char ucData = 0;
// Load up transmit msg
ds3231msg[0].addr = 0x34;
ds3231msg[0].flags = 0;
ds3231msg[0].len = sizeof(ucIndex);
ds3231msg[0].buf = &ucIndex;
// Load up receive msg
ds3231msg[1].addr = 0x34;
ds3231msg[1].flags = I2C_M_RD;
ds3231msg[1].len = sizeof(ucData);
ds3231msg[1].buf = &ucData;
// Load up i2c_rdwr_ioctl_data
struct i2c_rdwr_ioctl_data i2c_data; // declare our i2c_rdwr_ioctl_data structure
i2c_data.msgs = ds3231msg;
i2c_data.nmsgs = 2;

// Open file descriptor to I2C bus
int fd = open("/dev/i2c-0",O_RDWR);
if(fd<0) {
printf("Failed to open i2c-0.");
return -1;
}
// With our file descriptor, perform I2C message transfers
int result = ioctl(fd,I2C_RDWR,&i2c_data);
if(result < 0)
{
printf("ioctl error: %s\n", strerror(errno));
return -1;
}
// We have read a data byte from index 2... Display it
printf("Index 2: 0x%02X\n",ucData);
close(fd);

char* data = (char*)&i2c_data;
for (int a = 0; a < sizeof(i2c_data); a++) {
   printf("%02X ",*(data+a));
}

printf("\n");

char* data2 = (char*)&ds3231msg;
for (int a = 0; a < sizeof(ds3231msg); a++) {
   printf("%02X ",*(data2+a));
}

printf("\n");
printf("%i\n", sizeof(data));

return 0;
} 
