# Stelarc 

**Stelarc** est une application graphique moderne de compression et d'extraction, inspirée des outils populaires, mais avec des fonctionnalités avancées et un support pour des technologies comme **Precomp**, **SREP**,  **LOLZ**, et d'autres algorithmes de compression.

## Fonctionnalités

 **🔄 CRC (Cyclic Redundancy Check)** :
- Vérification d'intégrité : Utilisez le CRC pour vérifier l'intégrité des données et détecter les erreurs de transmission.
- Support pour BLAKE3 SHA3-256..  : Implémentation rapide et efficace pour le calcul des checksums CRC32, BLAKE3, MD5, SHA-256, SHA3-256

- **📁Explorateur de fichiers intégré** :
  - Navigation intuitive dans les répertoires.
  - Historique de navigation avec les boutons "Retour" et "Avancer".
  - Sélection facile de fichiers et dossiers pour la compression ou l'extraction.

- **🗜Compression avancée** :
  - Support des technologies comme **Precomp**, **SREP**, et autres.
  - Plusieurs presets de compression disponibles pour répondre à différents besoins :
    - Instantané
    - Optimisé pour les disques durs
    - Compression rapide
    - Compression normale
    - Compression  avec précompression et LZMA
    - Compression  avec précompression et Lolz
    - Compression  avec précompression et Srep

- **🔍Extraction puissante** :
  - Gestion des archives complexes.
  - Sélection du dossier de destination pour une extraction personnalisée.

- **Interface utilisateur moderne** :
  - Basée sur [egui](https://github.com/emilk/egui), offrant une expérience fluide et réactive.
  - Journaux en temps réel pour suivre les actions effectuées.
![image](https://github.com/user-attachments/assets/22c1a823-787e-4d35-a0be-ee165deaf0e6)


## Prérequis

- **Rust** : Assurez-vous que Rust est installé sur votre machine. Si ce n'est pas le cas, installez-le via [rustup](https://rustup.rs/).

## Installation

1. Clonez ce dépôt :
   ```sh
   git clone https://github.com/Luxuse/stelarc.git
   cd stelarc
