//! Учётные записи одной машины. Пароли хранятся открытым текстом — это игра,
//! и пароли нужны генератору (слабые пароли, словари) и подбору.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct User {
    pub name: String,
    pub password: String,
    pub uid: u32,
    pub home: String,
    pub shell: String,
    /// Может выполнять `sudo`.
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct UserDb {
    users: Vec<User>,
    next_uid: u32,
}

/// Домашняя папка по имени пользователя.
pub fn home_for(name: &str) -> String {
    if name == "root" {
        "/root".to_string()
    } else {
        format!("/home/{}", name)
    }
}

impl UserDb {
    /// База с одним root.
    pub fn new_root(root_password: &str) -> UserDb {
        UserDb {
            users: vec![User {
                name: "root".to_string(),
                password: root_password.to_string(),
                uid: 0,
                home: "/root".to_string(),
                shell: "/bin/sh".to_string(),
                is_admin: true,
            }],
            next_uid: 1000,
        }
    }

    /// Добавить пользователя (существующему — сменить пароль и права).
    pub fn add(&mut self, name: &str, password: &str, is_admin: bool) {
        if let Some(u) = self.users.iter_mut().find(|u| u.name == name) {
            u.password = password.to_string();
            u.is_admin = is_admin || u.name == "root";
            return;
        }
        let uid = if self.next_uid == 0 { 1000 } else { self.next_uid };
        self.next_uid = uid + 1;
        self.users.push(User {
            name: name.to_string(),
            password: password.to_string(),
            uid,
            home: home_for(name),
            shell: "/bin/sh".to_string(),
            is_admin,
        });
    }

    /// Удалить пользователя; root удалить нельзя.
    pub fn remove(&mut self, name: &str) -> bool {
        if name == "root" {
            return false;
        }
        let before = self.users.len();
        self.users.retain(|u| u.name != name);
        self.users.len() != before
    }

    pub fn get(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|u| u.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut User> {
        self.users.iter_mut().find(|u| u.name == name)
    }

    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn check_password(&self, name: &str, password: &str) -> bool {
        self.get(name).map(|u| u.password == password).unwrap_or(false)
    }

    pub fn set_password(&mut self, name: &str, password: &str) -> bool {
        match self.get_mut(name) {
            Some(u) => {
                u.password = password.to_string();
                true
            }
            None => false,
        }
    }

    pub fn is_root(&self, name: &str) -> bool {
        name == "root"
    }

    /// root или помеченный администратором.
    pub fn is_admin(&self, name: &str) -> bool {
        name == "root" || self.get(name).map(|u| u.is_admin).unwrap_or(false)
    }

    pub fn names(&self) -> Vec<String> {
        self.users.iter().map(|u| u.name.clone()).collect()
    }

    pub fn list(&self) -> &[User] {
        &self.users
    }

    pub fn len(&self) -> usize {
        self.users.len()
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    /// Домашняя папка пользователя (или по правилу, если такого нет).
    pub fn home_of(&self, name: &str) -> String {
        self.get(name).map(|u| u.home.clone()).unwrap_or_else(|| home_for(name))
    }

    /// Содержимое `/etc/passwd` по текущему списку.
    pub fn passwd_text(&self) -> String {
        let mut s = String::new();
        for u in &self.users {
            s.push_str(&format!("{}:x:{}:{}:{}:{}:{}\n", u.name, u.uid, u.uid, u.name, u.home, u.shell));
        }
        s
    }
}
