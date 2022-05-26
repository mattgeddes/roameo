# roameo
A simple tool for matching network attributes, such as ESSID or subnet.

Primarily as a way to be able to use ```Match exec``` in ```ssh_config(5)``` to define different OpenSSH client configurations for hosts based on current client-side network state. This is especially useful on laptops where when connected to the corporate Wi-Fi at the office, you may ssh direct to nodes, whereas outside the office you may need to include a jump host configuration. With ```roameo```, you define both configurations in your ```ssh_config(5)``` file(s) and wrap them with a ```Match exec``` to call ```roameo```. You can then seamlessly ssh to hosts with or without the jump host as needed.

There may be other use cases too.

This could easily have been implemented in shell or by doing ```fork()/exec()``` of command line tools, but I was looking for an excuse to write some Rust code.

TODO: Check the code in :)
