# Servant Project Outline

## Goal

To create a modified version of servo example broswer the can fetch data from autonomi. URL with the protocpl listed as ant eg "ant://<ADDRESS>" will fetch data from autonomi and display it in the browser id possible.

## Naming

The porject and application will be called "Servant" and the executable will be "servant".

## Core constraints

- Should produce a single executable that does not require any other files to run.
- Autonomi functions should come from the ant-core crate, however some highler level concepts such as directories need to be implemented in line with the ant-cli app.
- The locally cloned autonomi crates (ant-core and ant-cli) should not be altered locally though they will be updated from upstream repositories.

## Considerations regarding servo

- While this will be a seprate project it will be essentailly a modifed version of the servo example project. As this is regularly updated we want to structure the project in a way that we can easily pull in the latest changes from the servo example project into this project. Can you provide recommendation to achieve this?
- We don't need to make any major changes to the UI of the browser. We can use the existing UI of the servo example project.


## Considerations regarding autonomi

- As we will need to use the ant-core crate plus some additonal behavoir from the ant-cli crate which are also both under heaver development we also need to consder how to handle updates to these crates.
- We will need to follow the example of how client connections are made from the ant-cli project including the use of local config files...but we need a way for the configuration file to be created if they don't exist.
- We will need to follow the example of how directories are handled from the ant-cli project and use the same manifest format.
- As the ant-cli project is under development we made need to review it after updates in case we need to update our code to match the new behavior regarding the two previous points.
- It is possible to run a local testing network for autonomi and it would be nice to be able to easily switch between this and the main network though the main network should be the default.

## How autnomi url should be handled

- The urls should function weather input in the address bar or as hyperlinks or any other similar behaviour.
- the url will be begin with ant://
- then it will have an autonomi address which must be a 32 bit hexadecimal number if this is not the case the url should be rejected with an appropriate error.
- this may be followed by a path which could either indicate the filename if it is just a file or a path within the directory if it is a directory.
- Taking note of how ant-cli does this the hexadimal address should be attempted to retreive from autnomi and check in this order
    - if it is a diretory if so then all the consituents parts of the directory should be retreived
    - if its a file then the file should be retrieved
    - if it is just data that should be retrieved
    - if it is just a chunk this should be retrieved
    - if it is not possible to retreive the address it should display a user friendly error 
- If it is a directory and there is a sub path then this subpath should be retrieved from the directory contents and displayed in the browser.
- If it is a directory without a sub path and it has a index file then the index file should be displayed in the browser.
- If it is a directory without a sub path and it has no index file then it should display a list of all the files and directories in the directory.
- If it is a file and the sub path is a file name its should be given that name.
- If it is a file and there is no sub path parse the file to see if it is html and if so call it index.html and display it in the browser.
    - Poetnaily if possible t infer other commin file types we might want to do this as well.
- If it is a data or chunk of data that cannot be parsed as html, pdf, image or any other know file type display the raw date as text in the browser.

## UI modifications

- Currently the autonomi network is very slow so we will want to give some feedback to users while loading. We will probalby both want to indicated how the connection is going and then the progress of retriveing the data (how many chunks retreived etc)...the ant-cli project gives some idea of how this might be done.
- While not essentially can we consider how diffuclt modifying the color scheme of the browser would be.

## Caching

- As the netowrk is slow is would be good to cache data locally can we consider the best way to do this. The ant-core crate does include structures for cacihg chunks locally...but given files are directories are more likely for webpages it may be better to cache as actually directories and files but we need to consder how we link this back to there address for recall. MAyebe we would want to do both that and chunk caching where the data is not files/directories.

## Planning

- I suspect the most crucial part of the planning will be how and if we can inject the functions of the autonomi protocl into the existing browser structure without needing to modify it to much. Can you come up with a plan for how we can do this and let me know if you think this is a feasible approach?

## Compilation

- Note servo has its own build system and uses a tool called mach to build the project.
- Will cross complicatio still be viable because of this?
- Target os would be linux, android, windows and macos.