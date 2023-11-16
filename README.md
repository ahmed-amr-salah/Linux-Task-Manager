# About This Project:
This project is linux task manager application that consists of two main
components: a command line-based part and a graphical interface part. The command
line-based part provides real-time monitoring, command execution, system information
retrieval, process management functionalities such as killing, searching, and filtering
processes. On the other hand, the graphical interface part offers a visually appealing
representation of system information, utilizing graphs and a tree view. It also displays
the currently running processes and their respective CPU usage .
The integration of both parts aims to cater to a wide range of users, accommodating
both experts who prefer the command line interface (CLI) and new users who are more
comfortable with graphical user interfaces (GUI). By providing a dual-interface
solution, the task manager application ensures that all users can leverage its capabilities
efficiently, regardless of their preferred interaction style.

# Screenshots:
![Screenshot from 2023-05-21 10-15-16](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/bc4bf325-efa5-4681-945d-5f1369e1fc0c)
![Screenshot from 2023-05-21 10-15-23](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/8cea7fb8-56dd-4b3a-baa9-ffce335551b4)
![Screenshot from 2023-05-21 10-16-37](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/5032b153-d887-4623-9b9e-9fc25a1694f5)
![Screenshot from 2023-05-21 10-16-52](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/d93e7457-6583-416f-ab3d-df512a23759e)
![Screenshot from 2023-05-21 10-16-55](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/2d104608-3ed9-4272-85a1-58d19a3a1f94)
![Screenshot from 2023-05-21 10-16-59](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/4869a05a-dc38-4edf-8f86-6a3a2bab0c0b)
![Screenshot from 2023-05-21 10-17-16](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/7459705a-ace4-4680-ab0a-57f5a4620594)
![Screenshot from 2023-05-21 10-17-23](https://github.com/Aahmedamr221201/Task-Manager/assets/90650191/86d59f38-d2c4-4b71-8a97-817c9c0ef085)


# User Guide: 

Command to run in cursive view:
- "show full table" -> Display a real-time updated processes table & system information.
- Search options:
	- "search -user [username]" -> Search for processes of a specific user.
	- "search -uid [process uid]" -> Search for processes given a specific user id.
	- "search -pid [process id]" -> Search for processes given a specific pid.
	- "search -ppid [process parent id] -> Search for processes given a speciific pid.
	- "search -name [process name] -> Search for processes given a name.
- Filter options:
	- "filter -[greater,less] -cpu -value -> Filter based on cpu% usage.
	- "filter -[greater,less] -mem -value -> Filter based on memory% usage.
	- "filter -[greater,less] -threads -value -> Filter based on threads count.
	- "filter -[range] -cpu [minimum] [maximum] -> Filter based on range for cpu usage.
	- "filter -[range] -mem [minimum] [maximum] -> Filter based on range for memory usage.
	- "filter -[range] -threads [minimum] [maximum] -> Filter based on range for threads count.
	- "filter -[range] -fd [minimum] [maximum] -> Filter based on range for open files count.
- "kill -pid [process pid] -> To kill a process based on pid.
- "show -help" -> To display all available command.

- For sorting, click on column's header.
# Dependencies:
+ **sysinfo crate: This crate provides a set of functions and structures to
gather system information, allowing the project to access and utilize
relevant system data .**

+ **procfc: This crate offers facilities to interact with process control
groups (cgroups) in the Linux kernel. It enables the project to manage
and monitor resource usage for individual processes or groups of
processes .**

+ **fltk: The fltk crate provides bindings to the FLTK (Fast Light Toolkit)
library, which is a cross-platform graphical user interface (GUI)toolkit. It enables the project to create a visually appealing and user-
friendly interface for the task manager.**

+ **cursive: This crate is a fully featured, text-based user interface (TUI)
library for Rust. It allows the project to implement a command-line
interface with interactive elements, enabling users to interact with the
task manager through text-based commands and menus.**

# Tech Stack:

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)

# Note:
We've included both executable files for both GUI and CLI views integrated together. However, when we run the overall executable file, only the CLI view opens.
However, running the code from the main project (CLI) while including the GUI executable works and both views open at the same time.
