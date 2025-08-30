//! Blimp is a simple package manager for Unix systems.

mod confirm;
mod install;
#[cfg(feature = "network")]
mod remote;
mod remove;
#[cfg(feature = "network")]
mod update;

use clap::{Args, Parser, Subcommand};
use common::{
	anyhow::{anyhow, Result},
	tokio, Environment,
};
use install::install;
use remove::remove;
use std::{env, path::PathBuf, process::exit};

#[derive(Args, Clone, Debug)]
struct PkgList {
	/// Packages
	packages: Vec<String>,
}

#[derive(Clone, Debug, Subcommand)]
enum Action {
	/// Synchronizes packages information from remotes
	#[cfg(feature = "network")]
	Update,
	/// Prints information about the given package(s)
	Info(PkgList),
	/// Installs the given package(s)
	Install(PkgList),
	/// Upgrades the given package(s). If no package is specified, the package manager updates
	/// every package that is not up-to-date
	Upgrade(PkgList),
	/// Removes the given package(s)
	Remove(PkgList),
	/// Cleans the cache
	Clean,
	/// Lists remote servers
	#[cfg(feature = "network")]
	RemoteList,
	/// Adds a remote server
	#[cfg(feature = "network")]
	RemoteAdd { remote: String },
	/// Removes a remote server
	#[cfg(feature = "network")]
	RemoteRemove { remote: String },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, after_long_help = "Environment variables:
\tSYSROOT: Specifies the path to the system's root
\tLOCAL_REPO: Specifies paths separated by `:` at which packages are stored locally (the SYSROOT variable does not apply to these paths)

All environment variables are optional")]
struct Cli {
	#[command(subcommand)]
	action: Action,
	/// The branch to use on package repositories, defaults to `stable`
	#[arg(short, long)]
	branch: Option<String>,
	/// The architecture to install for, defaults to the current
	#[arg(short, long)]
	arch: Option<String>,
}

async fn main_impl() -> Result<()> {
	let args = Cli::parse();
	let sysroot = env::var_os("SYSROOT")
		.map(PathBuf::from)
		.unwrap_or(PathBuf::from("/"));
	let local_repos = env::var("LOCAL_REPO") // TODO var_os
		.map(|s| s.split(':').map(PathBuf::from).collect())
		.unwrap_or_default();
	let mut env = Environment::acquire(&sysroot, local_repos, args.branch, args.arch)?
		.ok_or_else(|| anyhow!("failed to acquire lockfile"))?;
	match args.action {
		#[cfg(feature = "network")]
		Action::Update => update::update(&mut env).await?,
		Action::Info(_names) => todo!(),
		Action::Install(names) => install(&names.packages, &mut env).await?,
		Action::Upgrade(_names) => todo!(),
		Action::Remove(names) => remove(&names.packages, &mut env)?,
		Action::Clean => todo!(),
		#[cfg(feature = "network")]
		Action::RemoteList => remote::list(&env).await?,
		#[cfg(feature = "network")]
		Action::RemoteAdd {
			remote,
		} => remote::add(&mut env, remote)?,
		#[cfg(feature = "network")]
		Action::RemoteRemove {
			remote,
		} => remote::remove(&mut env, remote)?,
	}
	Ok(())
}

#[tokio::main]
async fn main() {
	if let Err(e) = main_impl().await {
		eprintln!("error: {e}");
		exit(1);
	}
}
