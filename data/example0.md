Czynności elementarne mapowane są na `Task`
- wykonanie skryptu/polecenia
- kopiowanie/usuwanie/tworzenie pliku (automatycznie wypełniając pliki template - *.tpl)

(komentarze // do usunięcia)
paramertry tasków pokazać jako np. adnotacje lub obiekty data 

```
    proc add_user(user) { // to jest zwijany task
        run_script('./smb_before_create_user.sh') // wenętrzne taski połączone sequence flow kolejno
        run_command('samba-tool create user ${user.username}') 
        file_copy('./default_profile/**/*', '/storage/profiles/${user.username}/', { user: user })
    }
```

Wszystkie czynności wykonywane są na hostach mapowanych na Pool

```
    @default_host: 'ad01.example.org` // Pool
    proc add_user(user) { // zwinięty task
        /* tasks */ 
    }
```

Jednostką kompilacji jest moduł

```
mod active_directory { // to jest nazwa całego diagramu

    @default_host: 'ad01.example.org` // Pool
    proc add_user(user) { // zwinięty task
        run_script('./smb_before_create_user.sh') // wenętrzne taski połączone sequence flow kolejno
        run_command('samba-tool create user ${user.username}') 
        file_copy('./default_profile/**/*', '/storage/profiles/${user.username}/', { user: user })
    }
    
    on add_user { user } => { //task wyzwalany zdarzeniem o nazwie add_user
        exec add_user(user) on `ad01.example.org`  // to jest wywolanie procedury add_user (połączenie poprzez message flow)
    }
}
```