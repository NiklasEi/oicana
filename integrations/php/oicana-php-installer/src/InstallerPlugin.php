<?php

declare(strict_types=1);

namespace Oicana\Installer;

use Composer\Composer;
use Composer\EventDispatcher\EventSubscriberInterface;
use Composer\IO\IOInterface;
use Composer\Plugin\PluginInterface;
use Composer\Script\Event;
use Composer\Script\ScriptEvents;

/**
 * Composer plugin that automatically downloads the Oicana native extension.
 *
 * This plugin hooks into Composer's post-install and post-update events
 * to download the appropriate native extension binary for the current platform.
 */
final class InstallerPlugin implements PluginInterface, EventSubscriberInterface
{
    /**
     * {@inheritdoc}
     */
    public function activate(Composer $composer, IOInterface $io): void
    {
    }

    /**
     * {@inheritdoc}
     */
    public function deactivate(Composer $composer, IOInterface $io): void
    {
    }

    /**
     * {@inheritdoc}
     */
    public function uninstall(Composer $composer, IOInterface $io): void
    {
    }

    /**
     * {@inheritdoc}
     *
     * @return array<string, string|array{0: string, 1?: int}>
     */
    public static function getSubscribedEvents(): array
    {
        return [
            ScriptEvents::POST_INSTALL_CMD => 'installExtension',
            ScriptEvents::POST_UPDATE_CMD => 'installExtension',
        ];
    }

    /**
     * Install the Oicana native extension.
     *
     * This method is called after `composer install` or `composer update`.
     */
    public function installExtension(Event $event): void
    {
        $io = $event->getIO();

        try {
            $io->write('<info>Installing Oicana native extension...</info>');

            $detector = new PlatformDetector();
            $platform = $detector->detect();

            $io->write(sprintf(
                '<info>Detected platform: %s</info>',
                $platform->getDescription()
            ));

            $downloader = new BinaryDownloader();
            $extensionPath = $downloader->download($platform);

            $io->write('<info>✓ Extension downloaded to: ' . $extensionPath . '</info>');

            $this->showInstallationInstructions($io, $extensionPath, $platform);

        } catch (\Throwable $e) {
            $io->writeError('<error>Failed to install Oicana extension: ' . $e->getMessage() . '</error>');
            $io->writeError('');
            $io->writeError('<comment>Manual Installation:</comment>');
            $io->writeError('<comment>You can download the extension manually from:</comment>');
            $io->writeError('<comment>https://github.com/oicana/oicana/releases</comment>');
            $io->writeError('');
        }
    }

    /**
     * Show instructions for enabling the extension in PHP.
     */
    private function showInstallationInstructions(IOInterface $io, string $extensionPath, Platform $platform): void
    {
        $io->write('');
        $io->write('<comment>To use the extension, add it to your php.ini:</comment>');

        $relativePath = $this->getRelativePath(getcwd(), $extensionPath);
        $io->write(sprintf('<comment>  extension=%s</comment>', $relativePath ?: $extensionPath));

        $io->write('');
        $io->write('<comment>Or load it at runtime:</comment>');
        $io->write(sprintf("<comment>  <?php dl('%s'); ?></comment>", $platform->getBinaryName()));

        $io->write('');
        $io->write('<comment>Verify installation with:</comment>');
        $io->write('<comment>  php -m | grep oicana_native</comment>');
        $io->write('');
    }

    /**
     * Get relative path from base to target.
     *
     * @return string|null Relative path or null if not possible
     */
    private function getRelativePath(string $from, string $to): ?string
    {
        $from = rtrim($from, DIRECTORY_SEPARATOR);
        $to = rtrim($to, DIRECTORY_SEPARATOR);

        $fromParts = explode(DIRECTORY_SEPARATOR, $from);
        $toParts = explode(DIRECTORY_SEPARATOR, $to);

        $commonLength = 0;
        $minLength = min(count($fromParts), count($toParts));
        for ($i = 0; $i < $minLength; $i++) {
            if ($fromParts[$i] === $toParts[$i]) {
                $commonLength++;
            } else {
                break;
            }
        }

        $relativeParts = [];
        for ($i = $commonLength; $i < count($fromParts); $i++) {
            $relativeParts[] = '..';
        }
        for ($i = $commonLength; $i < count($toParts); $i++) {
            $relativeParts[] = $toParts[$i];
        }

        if (count($relativeParts) === 0) {
            return null;
        }

        return implode(DIRECTORY_SEPARATOR, $relativeParts);
    }
}
